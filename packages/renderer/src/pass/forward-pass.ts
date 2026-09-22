import { IRendererFrame } from "#/graph/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * Composites blended surfaces over the tonemapped frame, tested against the G-buffer's depth, as Base orders it.
 */
export class ForwardPass implements IRendererPass {
  public readonly name: string = "forward";

  public render({ renderer, camera, targets, forward }: IRendererFrame): void {
    renderer.setRenderTarget(targets.composite);
    renderer.render(forward, camera);
  }

  public dispose(): void {}
}
