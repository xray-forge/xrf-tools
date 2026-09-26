import { NodeMaterial, QuadMesh, RenderTarget, Texture, WebGPURenderer } from "three/webgpu";

import { createDepthWritingQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererFrameSize } from "#/pass/renderer-frame-size";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { ResolvedTarget } from "#/pass/resolved-target";
import { toSpatialUpscale, toSpatialUpscaleDepth } from "#/pass/spatial-upscale-pass.tsl";

/**
 * FSR 1: the frame as a mode that does not jitter finishes it, upscaled to the output by EASU, with the drawn depth
 * nearest each pixel for the helpers to test against.
 */
export class SpatialUpscalePass implements IRendererPass {
  public readonly name: string = "fsr1";

  private readonly resolved: ResolvedTarget = new ResolvedTarget("fsr1");
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  /**
   * @param frame - The frame as drawn and smoothed, at the drawing's size.
   * @param targets - The frame's targets, whose depth the output carries upscaled.
   */
  public constructor(frame: Texture, targets: RendererTargets) {
    this.material = createDepthWritingQuadMaterial(
      toSpatialUpscale(frame),
      toSpatialUpscaleDepth(frame, targets.depth)
    );
    this.quad = new QuadMesh(this.material);
  }

  public get output(): RenderTarget {
    return this.resolved.output;
  }

  public resize(renderer: WebGPURenderer, { width, height }: IRendererFrameSize): void {
    this.resolved.resize(renderer, width, height);
  }

  public render({ renderer }: IRendererFrame): void {
    renderer.setRenderTarget(this.resolved.output);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.material.dispose();
    this.resolved.dispose();
  }
}
