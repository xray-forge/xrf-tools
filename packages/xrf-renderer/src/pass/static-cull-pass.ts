import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { StaticCull } from "#/scene/static/static-cull";

/**
 * Culls every static draw on the GPU for the view about to be drawn, before any pass draws one.
 */
export class StaticCullPass implements IRendererPass {
  public readonly name: string = "cull";

  private readonly cull: StaticCull;

  public constructor(cull: StaticCull) {
    this.cull = cull;
  }

  public render({ renderer }: IRendererFrame): void {
    this.cull.dispatch(renderer);
  }

  /** The cull is the scene's, which lets it go with its buffers. */
  public dispose(): void {}
}
