import { RenderTarget, WebGPURenderer } from "three/webgpu";

import { resizeBorrowedDepthTarget } from "#/internals/preserved-depth-target";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { toSharpened } from "#/pass/sharpen-pass.tsl";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
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
  private readonly draw: FullScreenDraw;

  /**
   * @param frame - The upscaled frame and its depth, at the output's size.
   * @param isDenoised - Whether RCAS spares what it finds noisy, as FSR 2's does.
   */
  public constructor(frame: RenderTarget, isDenoised: boolean) {
    this.output.texture.name = "rcas";
    this.output.depthTexture = frame.depthTexture;
    this.draw = new FullScreenDraw(
      createQuadMaterial(toSharpened(frame.texture, this.uniforms.strength, isDenoised)),
      this.output
    );
  }

  /**
   * @param sharpening - How sharp, from none to the most.
   */
  public setSharpening(sharpening: number): void {
    this.uniforms.apply(sharpening);
  }

  /** Sized after the upscaler, whose depth it shares and must neither clear nor free. */
  public resize(renderer: WebGPURenderer, { width, height }: IRendererFrameSize): void {
    resizeBorrowedDepthTarget(renderer, this.output, width, height);
  }

  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.draw(this.draw);
  }

  public render({ renderer }: IRendererFrame): void {
    this.draw.render(renderer);
  }

  /** Its colour goes with it; the depth is the upscaler's. */
  public dispose(): void {
    this.draw.dispose();
    this.output.dispose();
  }
}
