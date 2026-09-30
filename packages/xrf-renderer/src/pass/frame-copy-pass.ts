import { RenderTarget, Texture, WebGPURenderer } from "three/webgpu";

import { toFrameCopy } from "#/pass/frame-copy-pass.tsl";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";

/**
 * Copies a frame as it stands at one point of the frame into a target of its own, for a later pass to compare with.
 */
export class FrameCopyPass implements IRendererPass {
  public readonly name: string;
  /** The copy, at the drawing's size. */
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false });

  private readonly draw: FullScreenDraw;

  /**
   * @param name - What the frame report states it under, and the device labels its copy.
   * @param frame - What it copies.
   */
  public constructor(name: string, frame: Texture) {
    this.name = name;
    this.output.texture.name = name;
    this.draw = new FullScreenDraw(createQuadMaterial(toFrameCopy(frame)), this.output);
  }

  public resize(renderer: WebGPURenderer, { renderWidth, renderHeight }: IRendererFrameSize): void {
    this.output.setSize(renderWidth, renderHeight);
    renderer.initRenderTarget(this.output);
  }

  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.draw(this.draw);
  }

  public render({ renderer }: IRendererFrame): void {
    this.draw.render(renderer);
  }

  public dispose(): void {
    this.draw.dispose();
    this.output.dispose();
  }
}
