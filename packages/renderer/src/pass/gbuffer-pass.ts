import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IRendererFrame } from "#/graph/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * Fills the G-buffer with everything the deferred passes light.
 */
export class GBufferPass implements IRendererPass {
  public readonly name: string = "gbuffer";

  public render({ renderer, camera, targets, scenes }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(targets.gbuffer);
    renderer.clear(true, true, false);
    renderer.render(scenes[ERendererPass.DEFERRED], camera);
  }

  public dispose(): void {}
}
