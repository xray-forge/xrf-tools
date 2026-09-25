import { Nullable } from "@xrf/types";
import {
  DepthTexture,
  FloatType,
  HalfFloatType,
  LinearFilter,
  Matrix4,
  NodeMaterial,
  PerspectiveCamera,
  QuadMesh,
  RenderTarget,
  Texture,
  WebGPURenderer,
} from "three/webgpu";

import { initBorrowedDepthTarget } from "#/internals/borrowed-depth-target";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { ITemporalInputs, toResolvedDepth, toTemporalResolve } from "#/pass/temporal-antialias-pass.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { TemporalUniforms } from "#/uniforms/temporal-uniforms";

/** Jitter positions a drawn pixel cycles through at the output's size: eight of Halton (2, 3). */
const JITTER_SAMPLES: number = 8;

/** The Halton sequence's value at an index in a base, in `[0, 1)`. */
export function toHalton(index: number, base: number): number {
  let fraction: number = 1;
  let result: number = 0;

  for (let rest: number = index; rest > 0; rest = Math.floor(rest / base)) {
    fraction /= base;
    result += fraction * (rest % base);
  }

  return result;
}

/**
 * @param upscale - The ratio of the output's side to the drawing's.
 * @returns How many jitter positions the frames cycle through: FSR's eight times the ratio squared, so every output
 *   pixel a drawn pixel covers is sampled near its centre as often as at the output's size.
 */
export function toTemporalJitterPhases(upscale: number): number {
  return Math.ceil(JITTER_SAMPLES * upscale * upscale - 1e-6);
}

/**
 * @param phase - The frame's place in the cycle.
 * @returns The offset its samples are jittered by, in drawn pixels about the pixel's centre, `y` down.
 */
export function toTemporalJitter(phase: number): readonly [number, number] {
  return [toHalton(phase + 1, 2) - 0.5, toHalton(phase + 1, 3) - 0.5];
}

/**
 * @param output - A side of the output, in pixels.
 * @param upscale - The ratio of the output's side to the drawing's.
 * @returns That side of the scene as drawn.
 */
export function toRenderSize(output: number, upscale: number): number {
  return Math.max(1, Math.round(output / upscale));
}

/**
 * Offsets a perspective projection so the texel at `m` shows the scene at `m + 0.5 + jitter`: the image moves the other
 * way, by as many clip units. A perspective projection carries a clip offset times `w`, which is `-z`, in its third
 * column.
 *
 * @param projection - The projection, changed in place.
 * @param x - The jitter across, in pixels.
 * @param y - The jitter down, in pixels.
 * @param width - The drawing's width, in pixels.
 * @param height - And its height.
 */
export function jitterProjection(projection: Matrix4, x: number, y: number, width: number, height: number): void {
  projection.elements[8] += (2 * x) / Math.max(width, 1);
  projection.elements[9] -= (2 * y) / Math.max(height, 1);
}

/** One of the two history targets the resolve writes in turn, with the frame as shown and its depth beside it. */
function createResolveTarget(history: Texture, output: RenderTarget): RenderTarget {
  const target: RenderTarget = new RenderTarget(1, 1, { count: 2, depthBuffer: true });

  target.textures[0].dispose();
  target.textures[1].dispose();
  target.textures[0] = history;
  target.textures[1] = output.texture;
  target.depthTexture = output.depthTexture;

  return target;
}

/** A history texture: colour, and the distance along the view of what it showed, filtered for its reprojection. */
function createHistoryTexture(name: string): Texture {
  const target: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false, type: HalfFloatType });

  target.texture.name = name;
  target.texture.minFilter = LinearFilter;
  target.texture.magFilter = LinearFilter;
  target.texture.generateMipmaps = false;

  return target.texture;
}

/**
 * TAA: every scene pass draws with its samples jittered a fraction of a pixel, a different fraction each frame, and
 * this pass resolves the frame with its history, found through the motion every G-buffer surface writes. It stands
 * after the blended surfaces and before the helpers, which draw unjittered over what it resolved: the camera's
 * projection is restored once it has drawn.
 *
 * It upscales as TAAU: the scene is drawn smaller than the output, jittered within a drawn pixel through a longer cycle,
 * and resolved at the output's size, where a drawn sample landing near an output pixel's centre counts for more. The
 * output carries a depth of its own, the drawn depth nearest each pixel, which the helpers test against. FSR would take
 * the same inputs, the jitter included.
 */
export class TemporalAntialiasPass implements IRendererPass {
  public readonly name: string = "temporal";
  /** The frame as resolved, with the drawn depth at the output's size for the helpers drawn over it to test against. */
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true });

  private readonly quad: QuadMesh = new QuadMesh();
  private readonly temporal: TemporalUniforms = new TemporalUniforms();
  private readonly uniforms: RendererUniforms;
  private readonly histories: ReadonlyArray<Texture> = [
    createHistoryTexture("temporal-history-0"),
    createHistoryTexture("temporal-history-1"),
  ];
  private readonly resolves: ReadonlyArray<RenderTarget>;
  private readonly materials: ReadonlyArray<NodeMaterial>;
  /** The camera's projection as its controller left it, which the helpers draw with. */
  private readonly projection: Matrix4 = new Matrix4();
  private readonly projectionInverse: Matrix4 = new Matrix4();
  private jittered: Nullable<PerspectiveCamera> = null;
  private phase: number = 0;
  private phases: number = JITTER_SAMPLES;
  /** Which history this frame writes: the other holds the frame before. */
  private written: number = 0;
  private width: number = 0;
  private height: number = 0;
  /** The scene's size as drawn, which the jitter is measured in. */
  private renderWidth: number = 0;
  private renderHeight: number = 0;

  /**
   * @param targets - The frame's targets, whose tonemapped frame is resolved.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.uniforms = uniforms;
    this.output.texture.name = "temporal-output";
    this.output.depthTexture = new DepthTexture(1, 1, FloatType);
    this.output.depthTexture.name = "temporal-depth";
    this.resolves = this.histories.map((history: Texture) => createResolveTarget(history, this.output));
    this.materials = this.histories.map((_, index: number) => {
      const inputs: ITemporalInputs = {
        depth: targets.depth,
        frame: targets.scene.texture,
        history: this.histories[1 - index],
        motion: targets.motion,
      };
      const material: NodeMaterial = createQuadMaterial(
        toTemporalResolve(inputs, { camera: uniforms.camera, motion: uniforms.motion, temporal: this.temporal })
      );

      // Written wherever it stands, the test left off: three turns `AlwaysDepth` into `NeverDepth` for reversed depth.
      material.depthNode = toResolvedDepth(inputs, uniforms.motion);
      material.depthWrite = true;

      return material;
    });
  }

  /**
   * Sizes the history and output to the output and the jitter to the drawing, either change starting the history
   * afresh.
   *
   * @param renderer - The renderer the targets are drawn by.
   * @param width - The output's width, in device pixels.
   * @param height - And its height.
   * @param renderWidth - The scene's width as drawn.
   * @param renderHeight - And its height.
   */
  public resize(
    renderer: WebGPURenderer,
    width: number,
    height: number,
    renderWidth: number,
    renderHeight: number
  ): void {
    if (
      width === this.width &&
      height === this.height &&
      renderWidth === this.renderWidth &&
      renderHeight === this.renderHeight
    ) {
      return;
    }

    this.width = width;
    this.height = height;
    this.renderWidth = renderWidth;
    this.renderHeight = renderHeight;
    this.phases = toTemporalJitterPhases(width / Math.max(renderWidth, 1));
    this.phase = 0;
    this.output.setSize(width, height);
    this.resolves.forEach((target: RenderTarget) => target.setSize(width, height));
    // The resolve writes the depth the helpers then draw over: none of the three may clear it on its first draw.
    this.resolves.forEach((target: RenderTarget) => initBorrowedDepthTarget(renderer, target));
    initBorrowedDepthTarget(renderer, this.output);
    this.temporal.isHistoryValid.value = 0;
  }

  /**
   * Offsets the camera's projection by this frame's jitter, before anything reads it.
   *
   * @param camera - The drawing camera, its projection as its controller left it.
   */
  public jitter(camera: PerspectiveCamera): void {
    const [x, y] = toTemporalJitter(this.phase);

    this.projection.copy(camera.projectionMatrix);
    this.projectionInverse.copy(camera.projectionMatrixInverse);
    this.jittered = camera;
    this.uniforms.motion.jitter.value.set(x, y);
    jitterProjection(camera.projectionMatrix, x, y, this.renderWidth, this.renderHeight);
    camera.projectionMatrixInverse.copy(camera.projectionMatrix).invert();
  }

  public render({ renderer }: IRendererFrame): void {
    this.quad.material = this.materials[this.written];
    renderer.setRenderTarget(this.resolves[this.written]);
    this.quad.render(renderer);
    this.written = 1 - this.written;
    this.phase = (this.phase + 1) % this.phases;
    this.temporal.isHistoryValid.value = 1;
    this.restore();
  }

  public dispose(): void {
    this.restore();
    this.materials.forEach((material: NodeMaterial) => material.dispose());
    [...this.resolves, this.output].forEach((target: RenderTarget) => target.dispose());
    this.histories.forEach((history: Texture) => history.dispose());
    this.output.texture.dispose();
    this.output.depthTexture?.dispose();
  }

  /** Gives the camera back the projection its controller left it, for the helpers and the next frame. */
  private restore(): void {
    if (this.jittered) {
      this.jittered.projectionMatrix.copy(this.projection);
      this.jittered.projectionMatrixInverse.copy(this.projectionInverse);
      this.jittered = null;
    }
  }
}
