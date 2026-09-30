import { Nullable } from "@xrf/types";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { drawUnsorted } from "#/pass/unsorted-draw";
import { StaticCull } from "#/scene/static/static-cull";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { RainUniforms } from "#/uniforms/rain-uniforms";
import { STATIC_RAIN_VIEW } from "#/uniforms/static-draw-buffers";
import { RainCover } from "#/visibility/rain-cover";

/**
 * What stands over the rain, the level's shadow casters drawn straight down into the cover's depth: drawn again only
 * where the cover moved or what casts changed, and not at all while it does not rain.
 */
export class RainCoverPass implements IRendererPass {
  public readonly name: string = "rain-cover";

  private readonly casters: IStaticShadowCasters;
  private readonly cull: StaticCull;
  private readonly rain: RainUniforms;
  /** The casters' changes the cover was drawn at, or null before it was drawn at all. */
  private drawnVersion: Nullable<number> = null;

  /**
   * @param casters - What casts, which covers too.
   * @param cull - What culls the static draws, the cover's view among them.
   * @param rain - The rain's uniforms, which hold the cover.
   */
  public constructor(casters: IStaticShadowCasters, cull: StaticCull, rain: RainUniforms) {
    this.casters = casters;
    this.cull = cull;
    this.rain = rain;
  }

  /** Named whether or not it rains, so the first rain draws at once. */
  public listPipelines(pipelines: IRendererPipelines): void {
    pipelines.compute(this.cull.getViewKernels(STATIC_RAIN_VIEW));
  }

  public render({ renderer, viewCamera }: IRendererFrame): void {
    if (!this.rain.isFalling) {
      return;
    }

    const cover: RainCover = this.rain.cover;

    cover.fit(viewCamera.position);

    const isCulled: boolean = this.cull.cullView(renderer, STATIC_RAIN_VIEW, cover);

    if (!isCulled && this.drawnVersion === this.casters.shadowChanges.version) {
      return;
    }

    this.drawnVersion = this.casters.shadowChanges.version;
    renderer.setRenderTarget(this.rain.coverTarget);
    renderer.clear(false, true, false);
    drawUnsorted(renderer, () => {
      renderer.render(this.casters.shadowScenes[STATIC_RAIN_VIEW], cover.camera);

      if (this.casters.plainCasters.show(cover.planes)) {
        renderer.render(this.casters.plainCasters.scene, cover.camera);
      }
    });
    this.rain.commitCover();
  }

  /** The cover is the rain's, which lets it go with the uniforms. */
  public dispose(): void {}
}
