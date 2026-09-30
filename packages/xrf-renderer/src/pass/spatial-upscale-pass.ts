import { RenderTarget, Texture, WebGPURenderer } from "three/webgpu";

import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createDepthWritingQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { ResolvedTarget } from "#/pass/resolved-target";
import { toSpatialUpscale, toSpatialUpscaleDepth } from "#/pass/spatial-upscale-pass.tsl";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";

/**
 * FSR 1: the frame as a mode that does not jitter finishes it, upscaled to the output by EASU, with the drawn depth
 * nearest each pixel for the helpers to test against.
 */
export class SpatialUpscalePass implements IRendererPass {
  public readonly name: string = "fsr1";

  private readonly resolved: ResolvedTarget = new ResolvedTarget("fsr1");
  private readonly draw: FullScreenDraw;

  /**
   * @param frame - The frame as drawn and smoothed, at the drawing's size.
   * @param targets - The frame's targets, whose depth the output carries upscaled.
   */
  public constructor(frame: Texture, targets: RendererTargets) {
    this.draw = new FullScreenDraw(
      createDepthWritingQuadMaterial(toSpatialUpscale(frame), toSpatialUpscaleDepth(frame, targets.depth)),
      this.resolved.output
    );
  }

  public get output(): RenderTarget {
    return this.resolved.output;
  }

  public resize(renderer: WebGPURenderer, { width, height }: IRendererFrameSize): void {
    this.resolved.resize(renderer, width, height);
  }

  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.draw(this.draw);
  }

  public render({ renderer }: IRendererFrame): void {
    this.draw.render(renderer);
  }

  public dispose(): void {
    this.draw.dispose();
    this.resolved.dispose();
  }
}
