import { NodeMaterial, QuadMesh, RenderTarget, Texture, WebGPURenderer } from "three/webgpu";

import { toFrameCopy } from "#/pass/frame-copy-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererFrameSize } from "#/pass/renderer-frame-size";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * Copies a frame as it stands at one point of the frame into a target of its own, for a later pass to compare with.
 */
export class FrameCopyPass implements IRendererPass {
  public readonly name: string;
  /** The copy, at the drawing's size. */
  public readonly output: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false });

  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  /**
   * @param name - What the frame report states it under, and the device labels its copy.
   * @param frame - What it copies.
   */
  public constructor(name: string, frame: Texture) {
    this.name = name;
    this.output.texture.name = name;
    this.material = createQuadMaterial(toFrameCopy(frame));
    this.quad = new QuadMesh(this.material);
  }

  public resize(renderer: WebGPURenderer, { renderWidth, renderHeight }: IRendererFrameSize): void {
    this.output.setSize(renderWidth, renderHeight);
    renderer.initRenderTarget(this.output);
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
