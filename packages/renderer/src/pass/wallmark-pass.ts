import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IRendererFrame } from "#/graph/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * Composites wall marks into the G-buffer's albedo, tested against its depth, before any light reaches it: the
 * engine's `phase_wallmarks`, right after the scene fills the G-buffer.
 */
export class WallmarkPass implements IRendererPass {
  public readonly name: string = "wallmarks";

  public render({ renderer, camera, targets, scenes }: IRendererFrame): void {
    renderer.setRenderTarget(targets.wallmarks);
    renderer.render(scenes[ERendererPass.WALLMARK], camera);
  }

  public dispose(): void {}
}
