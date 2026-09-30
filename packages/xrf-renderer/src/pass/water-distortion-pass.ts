import { WebGPURenderer } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { FrameCopyPass } from "#/pass/frame-copy-pass";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { toWaterDistortionFragment } from "#/pass/water-distortion-pass.tsl";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * Moves what is seen through the water by what it wrote into the distortion target, once every composited surface is
 * down: the frame copied, then drawn again from the copy, each pixel read where the distortion moves it. Nothing at all
 * for a scene without water; in the frame only while the water is distorted, so its copy goes with it.
 */
export class WaterDistortionPass implements IRendererPass {
  public readonly name: string = "distortion";

  /** The frame as the blended surfaces left it, which the distortion reads from. */
  private readonly source: FrameCopyPass;
  private readonly draw: FullScreenDraw;

  /**
   * @param targets - What the frame draws into.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.source = new FrameCopyPass("distortion-source", targets.scene.texture);
    this.draw = new FullScreenDraw(
      createQuadMaterial(toWaterDistortionFragment(this.source.output.texture, targets.distortion, uniforms.water)),
      targets.scene
    );
  }

  public resize(renderer: WebGPURenderer, size: IRendererFrameSize): void {
    this.source.resize(renderer, size);
  }

  /** The copy too, which is its own. */
  public listPipelines(pipelines: IRendererPipelines): void {
    this.source.listPipelines(pipelines);
    pipelines.draw(this.draw);
  }

  public render(frame: IRendererFrame): void {
    const { renderer, scenes } = frame;

    if (!scenes[ERendererPass.WATER].children.length) {
      return;
    }

    this.source.render(frame);
    this.draw.render(renderer);
  }

  public dispose(): void {
    this.source.dispose();
    this.draw.dispose();
  }
}
