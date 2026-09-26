import { NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { toBackgroundMotion } from "#/pass/motion-background-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererFrameSize } from "#/pass/renderer-frame-size";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { MotionUniforms } from "#/uniforms/motion-uniforms";

/**
 * Writes the camera's motion into the motion target where the G-buffer drew nothing, so a temporal resolve reads the
 * sky's motion as it reads any surface's. In the frame only while a temporal mode resolves.
 */
export class MotionBackgroundPass implements IRendererPass {
  public readonly name: string = "motion-background";

  /** The motion target alone: the depth is read, not attached. */
  private readonly target: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false });
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  /**
   * @param targets - The frame's targets, whose motion is completed and whose depth tells where nothing was drawn.
   * @param motion - The motion uniforms.
   */
  public constructor(targets: RendererTargets, motion: MotionUniforms) {
    this.target.texture.dispose();
    this.target.texture = targets.motion;
    this.material = createQuadMaterial(toBackgroundMotion(targets.depth, motion));
    this.quad = new QuadMesh(this.material);
  }

  public resize(renderer: WebGPURenderer, { renderWidth, renderHeight }: IRendererFrameSize): void {
    this.target.setSize(renderWidth, renderHeight);
    renderer.initRenderTarget(this.target);
  }

  public render({ renderer }: IRendererFrame): void {
    renderer.setRenderTarget(this.target);
    this.quad.render(renderer);
  }

  /** The motion target is the G-buffer's, which frees it. */
  public dispose(): void {
    this.material.dispose();
  }
}
