import { NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { initPreservedDepthTarget } from "#/internals/preserved-depth-target";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererFrameSize } from "#/pass/renderer-frame-size";
import { IRendererPass } from "#/pass/renderer-pass";
import { toSharpened } from "#/pass/sharpen-pass.tsl";
import { SharpenUniforms } from "#/uniforms/sharpen-uniforms";

/**
 * RCAS over the upscaled frame, before the helpers draw over it: an upscaler softens what it reconstructs from fewer
 * samples.
 */
export class SharpenPass implements IRendererPass {
  public readonly name: string = "rcas";
  /** The sharpened frame, over the upscaled frame's depth for the helpers to test against. */
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: true });

  private readonly uniforms: SharpenUniforms = new SharpenUniforms();
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  /**
   * @param frame - The upscaled frame and its depth, at the output's size.
   * @param isDenoised - Whether RCAS spares what it finds noisy, as FSR 2's does.
   */
  public constructor(frame: RenderTarget, isDenoised: boolean) {
    this.output.texture.name = "rcas";
    this.output.depthTexture = frame.depthTexture;
    this.material = createQuadMaterial(toSharpened(frame.texture, this.uniforms.strength, isDenoised));
    this.quad = new QuadMesh(this.material);
  }

  /**
   * @param sharpening - How sharp, from none to the most.
   */
  public setSharpening(sharpening: number): void {
    this.uniforms.apply(sharpening);
  }

  /** Sized after the upscaler, whose depth it shares and must not clear. */
  public resize(renderer: WebGPURenderer, { width, height }: IRendererFrameSize): void {
    this.output.setSize(width, height);
    initPreservedDepthTarget(renderer, this.output);
  }

  public render({ renderer }: IRendererFrame): void {
    renderer.setRenderTarget(this.output);
    this.quad.render(renderer);
  }

  /** Its colour goes with it; the depth is the upscaler's. */
  public dispose(): void {
    this.material.dispose();
    this.output.dispose();
  }
}
