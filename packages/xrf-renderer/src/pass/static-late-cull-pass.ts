import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticCull } from "#/scene/static/static-cull";

/**
 * The second phase's cull: the depth the first phase drew reduced, and every static draw it hid culled again.
 */
export class StaticLateCullPass implements IRendererPass {
  public readonly name: string = "cull-late";

  private readonly targets: RendererTargets;
  private readonly cull: StaticCull;

  /**
   * @param targets - What the frame draws into, whose depth is reduced.
   * @param cull - What culls the static draws.
   */
  public constructor(targets: RendererTargets, cull: StaticCull) {
    this.targets = targets;
    this.cull = cull;
  }

  public render({ renderer }: IRendererFrame): void {
    this.cull.cullLate(renderer, this.targets.depth, this.targets.gbuffer.width, this.targets.gbuffer.height);
  }

  /** The cull is the scene's, which lets it go with its buffers. */
  public dispose(): void {}
}
