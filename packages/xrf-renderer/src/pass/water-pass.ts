import { RenderTarget, Scene } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { toWaterPrepareFragment } from "#/pass/water-pass.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * The water, composited over the lit frame before the forward surfaces, as the engine's first sorted forward draws it:
 * the depth behind it taken and the distortion target cleared first, then every water surface into the frame and the
 * distortion target at once. Nothing at all for a scene without water; in the frame only while the water is on.
 */
export class WaterPass implements IRendererScenePass {
  public readonly name: string = "water";
  public readonly isScenePass = true as const;
  public readonly scene: ERendererPass = ERendererPass.WATER;
  public readonly target: RenderTarget;

  /** The depth behind the water taken, and the distortion target cleared. */
  private readonly prepare: FullScreenDraw;

  /**
   * @param targets - What the frame draws into.
   * @param uniforms - What the frame's shaders read.
   */
  public constructor(targets: RendererTargets, uniforms: RendererUniforms) {
    this.target = targets.water;
    this.prepare = new FullScreenDraw(
      createQuadMaterial(toWaterPrepareFragment(targets.depth, uniforms.camera)),
      targets.waterPrepare
    );
  }

  /** Its own, apart from the water's surfaces, which compile as the scene's. */
  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.draw(this.prepare);
  }

  public render({ renderer, camera, scenes }: IRendererFrame): void {
    const scene: Scene = scenes[this.scene];

    if (!scene.children.length) {
      return;
    }

    this.prepare.render(renderer);
    renderer.setRenderTarget(this.target);
    renderer.render(scene, camera);
  }

  public dispose(): void {
    this.prepare.dispose();
  }
}
