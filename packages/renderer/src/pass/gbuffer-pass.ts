import { IRendererFrame } from "#/graph/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * Fills the G-buffer with everything the deferred passes light.
 */
export class GBufferPass implements IRendererPass {
  public readonly name: string = "gbuffer";

  public render({ renderer, camera, targets, deferred }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(targets.gbuffer);
    renderer.clear(true, true, false);
    renderer.render(deferred, camera);
  }

  public dispose(): void {}
}
