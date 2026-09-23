import { RenderTarget } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";

/**
 * Composites wall marks into the G-buffer's albedo, tested against its depth, before any light reaches it: the
 * engine's `phase_wallmarks`, right after the scene fills the G-buffer.
 */
export class WallmarkPass implements IRendererScenePass {
  public readonly name: string = "wallmarks";
  public readonly scene: ERendererPass = ERendererPass.WALLMARK;
  public readonly target: RenderTarget;

  public constructor(targets: RendererTargets) {
    this.target = targets.wallmarks;
  }

  public render({ renderer, camera, scenes }: IRendererFrame): void {
    renderer.setRenderTarget(this.target);
    renderer.render(scenes[this.scene], camera);
  }

  public dispose(): void {}
}
