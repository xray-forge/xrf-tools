import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticCull } from "#/scene/static/static-cull";

/**
 * Draws on into the G-buffer the static draws the second cull found seen past what the first phase drew.
 */
export class GBufferLatePass implements IRendererPass {
  public readonly name: string = "gbuffer-late";

  private readonly targets: RendererTargets;
  private readonly cull: StaticCull;

  /**
   * @param targets - What the frame draws into.
   * @param cull - What culls the static draws, whose second phase this draws.
   */
  public constructor(targets: RendererTargets, cull: StaticCull) {
    this.targets = targets;
    this.cull = cull;
  }

  public render({ renderer, camera }: IRendererFrame): void {
    renderer.setRenderTarget(this.targets.gbuffer);
    renderer.sortObjects = false;
    this.cull.drawLate(renderer, camera);
    renderer.sortObjects = true;
  }

  public dispose(): void {}
}
