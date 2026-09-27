import { NodeMaterial, QuadMesh, RenderTarget, WebGPURenderer } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { toFrameCopy } from "#/pass/frame-copy-pass.tsl";
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
 * while the water is not distorted or the scene has none.
 */
export class WaterDistortionPass implements IRendererPass {
  public readonly name: string = "distortion";

  private readonly targets: RendererTargets;
  private readonly copy: RenderTarget = new RenderTarget(1, 1, { depthBuffer: false });
  private readonly copyMaterial: NodeMaterial;
  private readonly copyQuad: QuadMesh;
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;
  private isDistorted: boolean = true;

  /**
   * @param targets - What the frame draws into.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.targets = targets;
    this.copy.texture.name = "distortion-source";
    this.copyMaterial = createQuadMaterial(toFrameCopy(targets.scene.texture));
    this.copyQuad = new QuadMesh(this.copyMaterial);
    this.material = createQuadMaterial(
      toWaterDistortionFragment(this.copy.texture, targets.distortion, uniforms.water)
    );
    this.quad = new QuadMesh(this.material);
  }

  /**
   * @param isDistorted - Whether the water moves what is seen through it.
   */
  public setDistorted(isDistorted: boolean): void {
    this.isDistorted = isDistorted;
  }

  public resize(renderer: WebGPURenderer, { renderWidth, renderHeight }: IRendererFrameSize): void {
    this.copy.setSize(renderWidth, renderHeight);
    renderer.initRenderTarget(this.copy);
  }

  public render({ renderer, scenes }: IRendererFrame): void {
    if (!this.isDistorted || !scenes[ERendererPass.WATER].children.length) {
      return;
    }

    renderer.setRenderTarget(this.copy);
    this.copyQuad.render(renderer);
    renderer.setRenderTarget(this.targets.scene);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.copyMaterial.dispose();
    this.material.dispose();
    this.copy.dispose();
  }
}
