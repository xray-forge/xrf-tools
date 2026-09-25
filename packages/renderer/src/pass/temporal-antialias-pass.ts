import { Nullable } from "@xrf/types";
import {
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

import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { toTemporalResolve } from "#/pass/temporal-antialias-pass.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { TemporalUniforms } from "#/uniforms/temporal-uniforms";

/** Jitter positions a pixel cycles through: eight of Halton (2, 3), each point of a pixel within a few frames. */
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

/** The offsets the samples are jittered by, in pixels about the pixel's centre, `y` down. */
export const TEMPORAL_JITTER: ReadonlyArray<readonly [number, number]> = Array.from(
  { length: JITTER_SAMPLES },
  (_, index: number) => [toHalton(index + 1, 2) - 0.5, toHalton(index + 1, 3) - 0.5] as const
);

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

/** One of the two history targets the resolve writes in turn, with the frame as shown beside it. */
function createResolveTarget(history: Texture, output: Texture): RenderTarget {
  const target: RenderTarget = new RenderTarget(1, 1, { count: 2, depthBuffer: false });

  target.textures[0].dispose();
  target.textures[1].dispose();
  target.textures[0] = history;
  target.textures[1] = output;

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
 * Laid out for upscaling: the frame is read at its own size and the history and output written at the output's, so
 * TAAU is this resolve with the scene drawn smaller and the jitter spanning an output pixel, and FSR takes the same
 * inputs, the jitter included.
 */
export class TemporalAntialiasPass implements IRendererPass {
  public readonly name: string = "temporal";
  /** The frame as resolved, with the G-buffer's depth for the helpers drawn over it to test against. */
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
  private sample: number = 0;
  /** Which history this frame writes: the other holds the frame before. */
  private written: number = 0;
  private width: number = 0;
  private height: number = 0;

  /**
   * @param targets - The frame's targets, whose tonemapped frame is resolved.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.uniforms = uniforms;
    this.output.texture.name = "temporal-output";
    this.output.depthTexture = targets.depth;
    this.resolves = this.histories.map((history: Texture) => createResolveTarget(history, this.output.texture));
    this.materials = this.histories.map((_, index: number) =>
      createQuadMaterial(
        toTemporalResolve(
          {
            depth: targets.depth,
            frame: targets.scene.texture,
            history: this.histories[1 - index],
            motion: targets.motion,
          },
          { camera: uniforms.camera, motion: uniforms.motion, temporal: this.temporal }
        )
      )
    );
  }

  /**
   * Sizes the history and output to the frame, which starts the history afresh. Before anything draws: the output
   * shares the G-buffer's depth, and allocating it reallocates that.
   *
   * @param renderer - The renderer the targets are drawn by.
   * @param width - Drawing buffer width, in device pixels.
   * @param height - Drawing buffer height, in device pixels.
   */
  public resize(renderer: WebGPURenderer, width: number, height: number): void {
    if (width === this.width && height === this.height) {
      return;
    }

    this.width = width;
    this.height = height;
    this.output.setSize(width, height);
    this.resolves.forEach((target: RenderTarget) => target.setSize(width, height));
    renderer.initRenderTarget(this.output);
    this.resolves.forEach((target: RenderTarget) => renderer.initRenderTarget(target));
    this.temporal.isHistoryValid.value = 0;
  }

  /**
   * Offsets the camera's projection by this frame's jitter, before anything reads it.
   *
   * @param camera - The drawing camera, its projection as its controller left it.
   */
  public jitter(camera: PerspectiveCamera): void {
    const [x, y] = TEMPORAL_JITTER[this.sample];

    this.projection.copy(camera.projectionMatrix);
    this.projectionInverse.copy(camera.projectionMatrixInverse);
    this.jittered = camera;
    this.uniforms.motion.jitter.value.set(x, y);
    jitterProjection(camera.projectionMatrix, x, y, this.width, this.height);
    camera.projectionMatrixInverse.copy(camera.projectionMatrix).invert();
  }

  public render({ renderer }: IRendererFrame): void {
    this.quad.material = this.materials[this.written];
    renderer.setRenderTarget(this.resolves[this.written]);
    this.quad.render(renderer);
    this.written = 1 - this.written;
    this.sample = (this.sample + 1) % TEMPORAL_JITTER.length;
    this.temporal.isHistoryValid.value = 1;
    this.restore();
  }

  public dispose(): void {
    this.restore();
    this.materials.forEach((material: NodeMaterial) => material.dispose());
    // The output only borrows the G-buffer's depth, which the G-buffer owns and frees.
    [...this.resolves, this.output].forEach((target: RenderTarget) => target.dispose());
    this.histories.forEach((history: Texture) => history.dispose());
    this.output.texture.dispose();
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
