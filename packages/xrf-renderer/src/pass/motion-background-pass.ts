import { NodeMaterial, QuadMesh, RenderTarget } from "three/webgpu";

import { toBackgroundMotion } from "#/pass/motion-background-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { MotionUniforms } from "#/uniforms/motion-uniforms";

/**
 * Writes the camera's motion into the motion target where the G-buffer drew nothing, so a temporal resolve reads the
 * sky's motion as it reads any surface's. In the frame only while a temporal mode resolves.
 */
export class MotionBackgroundPass implements IRendererPass {
  public readonly name: string = "motion-background";

  /** The motion target alone, the frame's targets' own: the depth is read, not attached. */
  private readonly target: RenderTarget;
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  /**
   * @param targets - The frame's targets, whose motion is completed and whose depth tells where nothing was drawn.
   * @param motion - The motion uniforms.
   */
  public constructor(targets: RendererTargets, motion: MotionUniforms) {
    this.target = targets.backgroundMotion;
    this.material = createQuadMaterial(toBackgroundMotion(targets.depth, motion));
    this.quad = new QuadMesh(this.material);
  }

  public render({ renderer }: IRendererFrame): void {
    renderer.setRenderTarget(this.target);
    this.quad.render(renderer);
  }

  /** The target is the frame's, which sizes and frees it. */
  public dispose(): void {
    this.material.dispose();
  }
}
