import {
  DepthTexture,
  FloatType,
  HalfFloatType,
  LinearFilter,
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
import { RendererTargets } from "#/pass/renderer-targets";
import { ITemporalInputs, toTemporalResolve } from "#/pass/temporal-antialias-pass.tsl";
import { TemporalJitter } from "#/pass/temporal-jitter";
import { ITemporalUpscaler } from "#/pass/temporal-upscaler";
import { toUpscaledDepth } from "#/pass/upscale-depth.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { TemporalUniforms } from "#/uniforms/temporal-uniforms";

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
 * output carries a depth of its own, the drawn depth nearest each pixel, which the helpers test against.
 */
export class TemporalAntialiasPass implements ITemporalUpscaler {
  public readonly name: string = "temporal";
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true });

  private readonly quad: QuadMesh = new QuadMesh();
  private readonly temporal: TemporalUniforms = new TemporalUniforms();
  private readonly jitters: TemporalJitter;
  private readonly histories: ReadonlyArray<Texture> = [
    createHistoryTexture("temporal-history-0"),
    createHistoryTexture("temporal-history-1"),
  ];
  private readonly resolves: ReadonlyArray<RenderTarget>;
  private readonly materials: ReadonlyArray<NodeMaterial>;
  /** Which history this frame writes: the other holds the frame before. */
  private written: number = 0;
  private width: number = 0;
  private height: number = 0;
  private renderWidth: number = 0;
  private renderHeight: number = 0;

  /**
   * @param targets - The frame's targets, whose tonemapped frame is resolved.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.jitters = new TemporalJitter(uniforms.motion);
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
      material.depthNode = toUpscaledDepth(inputs.frame, inputs.depth, uniforms.motion.jitter);
      material.depthWrite = true;

      return material;
    });
  }

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
    this.jitters.resize(width / Math.max(renderWidth, 1), renderWidth, renderHeight);
    this.output.setSize(width, height);
    this.resolves.forEach((target: RenderTarget) => target.setSize(width, height));
    // The resolve writes the depth the helpers then draw over: none of the three may clear it on its first draw.
    this.resolves.forEach((target: RenderTarget) => initBorrowedDepthTarget(renderer, target));
    initBorrowedDepthTarget(renderer, this.output);
    this.temporal.isHistoryValid.value = 0;
  }

  public jitter(camera: PerspectiveCamera): void {
    this.jitters.apply(camera);
  }

  public render({ renderer }: IRendererFrame): void {
    this.quad.material = this.materials[this.written];
    renderer.setRenderTarget(this.resolves[this.written]);
    this.quad.render(renderer);
    this.written = 1 - this.written;
    this.jitters.advance();
    this.temporal.isHistoryValid.value = 1;
    this.jitters.restore();
  }

  public dispose(): void {
    this.jitters.dispose();
    this.materials.forEach((material: NodeMaterial) => material.dispose());
    [...this.resolves, this.output].forEach((target: RenderTarget) => target.dispose());
    this.histories.forEach((history: Texture) => history.dispose());
    this.output.texture.dispose();
    this.output.depthTexture?.dispose();
  }
}
