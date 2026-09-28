import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { drawUnsorted } from "#/pass/unsorted-draw";
import { StaticCull } from "#/scene/static/static-cull";

/**
 * Clears the G-buffer and draws into it the static draws the first cull kept, before anything else: the depth the
 * second cull and the pyramid reduce holds the static draws alone.
 */
export class GBufferEarlyPass implements IRendererPass {
  public readonly name: string = "gbuffer-early";

  private readonly targets: RendererTargets;
  private readonly cull: StaticCull;

  /**
   * @param targets - What the frame draws into.
   * @param cull - What culls the static draws, whose first phase this draws.
   */
  public constructor(targets: RendererTargets, cull: StaticCull) {
    this.targets = targets;
    this.cull = cull;
  }

  public render({ renderer, camera }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(this.targets.gbuffer);
    renderer.clear(true, true, false);
    drawUnsorted(renderer, () => this.cull.drawEarly(renderer, camera));
  }

  public dispose(): void {}
}
