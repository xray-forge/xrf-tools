import { RenderTarget } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";

/**
 * Fills the G-buffer with everything the deferred passes light.
 */
export class GBufferPass implements IRendererScenePass {
  public readonly name: string = "gbuffer";
  public readonly scene: ERendererPass = ERendererPass.DEFERRED;
  public readonly target: RenderTarget;

  public constructor(targets: RendererTargets) {
    this.target = targets.gbuffer;
  }

  public render({ renderer, camera, scenes }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(this.target);
    renderer.clear(true, true, false);
    renderer.render(scenes[this.scene], camera);
  }

  public dispose(): void {}
}
