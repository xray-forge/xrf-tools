import { NodeMaterial, QuadMesh, RenderTarget, Scene } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { toWaterPrepareFragment } from "#/pass/water-pass.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * The water, composited over the lit frame before the forward surfaces, as the engine's first sorted forward draws it:
 * the depth behind it taken and the distortion target cleared first, then every water surface into the frame and the
 * distortion target at once. Nothing at all for a scene without water.
 */
export class WaterPass implements IRendererScenePass {
  public readonly name: string = "water";
  public readonly scene: ERendererPass = ERendererPass.WATER;
  public readonly target: RenderTarget;

  private readonly targets: RendererTargets;
  private readonly material: NodeMaterial;
  private readonly quad: QuadMesh;
  private isEnabled: boolean = true;

  /**
   * @param targets - What the frame draws into.
   * @param uniforms - What the frame's shaders read: the water's depth behind is pointed at the frame's.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.targets = targets;
    this.target = targets.water;
    this.material = createQuadMaterial(toWaterPrepareFragment(targets.depth, uniforms.camera));
    this.quad = new QuadMesh(this.material);
    uniforms.water.depth.value = targets.waterDepth;
  }

  /**
   * @param isEnabled - Whether the water is drawn.
   */
  public setEnabled(isEnabled: boolean): void {
    this.isEnabled = isEnabled;
  }

  public render({ renderer, camera, scenes }: IRendererFrame): void {
    const scene: Scene = scenes[this.scene];

    if (!this.isEnabled || !scene.children.length) {
      return;
    }

    renderer.setRenderTarget(this.targets.waterPrepare);
    this.quad.render(renderer);
    renderer.setRenderTarget(this.target);
    renderer.render(scene, camera);
  }

  public dispose(): void {
    this.material.dispose();
  }
}
