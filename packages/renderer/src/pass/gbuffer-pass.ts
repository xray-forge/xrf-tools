import { IRendererFrame } from "#/graph/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { ESurfacePass } from "#/scene/surface-material";

/**
 * Fills the G-buffer with everything the deferred passes light.
 */
export class GBufferPass implements IRendererPass {
  public readonly name: string = "gbuffer";

  public render({ renderer, camera, targets, scenes }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(targets.gbuffer);
    renderer.clear(true, true, false);
    renderer.render(scenes[ESurfacePass.DEFERRED], camera);
  }

  public dispose(): void {}
}
