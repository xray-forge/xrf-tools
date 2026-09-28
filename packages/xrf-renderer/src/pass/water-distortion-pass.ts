import { NodeMaterial, QuadMesh, WebGPURenderer } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { FrameCopyPass } from "#/pass/frame-copy-pass";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
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

  private readonly targets: RendererTargets;
  /** The frame as the blended surfaces left it, which the distortion reads from. */
  private readonly source: FrameCopyPass;
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;

  /**
   * @param targets - What the frame draws into.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.targets = targets;
    this.source = new FrameCopyPass("distortion-source", targets.scene.texture);
    this.material = createQuadMaterial(
      toWaterDistortionFragment(this.source.output.texture, targets.distortion, uniforms.water)
    );
    this.quad = new QuadMesh(this.material);
  }

  public resize(renderer: WebGPURenderer, size: IRendererFrameSize): void {
    this.source.resize(renderer, size);
  }

  public render(frame: IRendererFrame): void {
    const { renderer, scenes } = frame;

    if (!scenes[ERendererPass.WATER].children.length) {
      return;
    }

    this.source.render(frame);
    renderer.setRenderTarget(this.targets.scene);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.source.dispose();
    this.material.dispose();
  }
}
