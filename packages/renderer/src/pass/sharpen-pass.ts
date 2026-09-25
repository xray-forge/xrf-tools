import { NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { initBorrowedDepthTarget } from "#/internals/borrowed-depth-target";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { toSharpened } from "#/pass/sharpen-pass.tsl";
import { SharpenUniforms } from "#/uniforms/sharpen-uniforms";

/**
 * Sharpens the upscaled frame before the helpers draw over it: TAAU softens what it reconstructs from fewer samples.
 * In the frame only while TAA upscales and the sharpening is above none.
 */
export class SharpenPass implements IRendererPass {
  public readonly name: string = "sharpen";
  /** The sharpened frame, with the resolved depth for the helpers to test against. */
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true });

  private readonly uniforms: SharpenUniforms = new SharpenUniforms();
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  /**
   * @param frame - The resolved frame and its depth, at the output's size.
   */
  public constructor(frame: RenderTarget) {
    this.output.texture.name = "sharpened";
    this.output.depthTexture = frame.depthTexture;
    this.material = createQuadMaterial(toSharpened(frame.texture, this.uniforms.strength));
    this.quad = new QuadMesh(this.material);
  }

  /**
   * @param sharpening - How sharp, from none to the most.
   */
  public setSharpening(sharpening: number): void {
    this.uniforms.apply(sharpening);
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
    // The resolve's depth, which the helpers draw over after this: the first draw into it must not clear it.
    initBorrowedDepthTarget(renderer, this.output);
  }

  public render({ renderer }: IRendererFrame): void {
    renderer.setRenderTarget(this.output);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
    this.output.dispose();
  }
}
