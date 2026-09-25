import { DepthTexture, FloatType, NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { initBorrowedDepthTarget } from "#/internals/borrowed-depth-target";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { toSpatialUpscale } from "#/pass/spatial-upscale-pass.tsl";
import { toUpscaledDepth } from "#/pass/upscale-depth.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * FSR 1: the frame as the non-temporal modes finish it, upscaled to the output by EASU, with the drawn depth nearest
 * each pixel for the helpers to test against. In the frame only while those modes draw the scene smaller.
 */
export class SpatialUpscalePass implements IRendererPass {
  public readonly name: string = "upscale";
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true });

  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  /**
   * @param frame - The frame as drawn and smoothed, at the drawing's size.
   * @param targets - The frame's targets, whose depth the output carries upscaled.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(frame: RenderTarget, targets: RendererTargets, uniforms: RendererUniforms) {
    this.output.texture.name = "upscaled";
    this.output.depthTexture = new DepthTexture(1, 1, FloatType);
    this.output.depthTexture.name = "upscaled-depth";
    this.material = createQuadMaterial(toSpatialUpscale(frame.texture));
    // Written wherever it stands, the test left off: three turns `AlwaysDepth` into `NeverDepth` for reversed depth.
    this.material.depthNode = toUpscaledDepth(frame.texture, targets.depth, uniforms.motion.jitter);
    this.material.depthWrite = true;
    this.quad = new QuadMesh(this.material);
  }

  /**
   * @param renderer - The renderer the target is drawn by.
   * @param width - The output's width, in device pixels.
   * @param height - And its height.
   */
  public resize(renderer: WebGPURenderer, width: number, height: number): void {
    if (width === this.output.width && height === this.output.height) {
      return;
    }

    this.output.setSize(width, height);
    // It writes the depth the helpers then draw over: the first draw into it must not clear it.
    initBorrowedDepthTarget(renderer, this.output);
  }

  public render({ renderer }: IRendererFrame): void {
    renderer.setRenderTarget(this.output);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
    this.output.dispose();
    this.output.depthTexture?.dispose();
  }
}
