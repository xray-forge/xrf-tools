import { RefObservable, runInAction } from "@wirestate/mobx";
import {
  EMPTY_RENDER_FRAME_COST,
  EMPTY_RENDERER_PASS_TIMINGS,
  IRendererPassTimings,
  IRendererReport,
  IRenderFrameCost,
  NEUTRAL_RENDERER_LIGHTING,
} from "@xrf/renderer";

import { IRenderLighting, toRendererLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderSurfaceService } from "@/core/render/lib/surface/render-surface-service";

/**
 * A renderer drawing one asset against a backdrop, which the texture and visual viewers both are: under a neutral
 * light, with what its frames cost read out over them.
 */
export abstract class RenderAssetService extends RenderSurfaceService {
  /** What frames are costing, for whatever draws the readout over them. */
  @RefObservable()
  public frameCost: IRenderFrameCost = EMPTY_RENDER_FRAME_COST;

  /** What each pass of them cost on the GPU, for the same readout. */
  @RefObservable()
  public timings: IRendererPassTimings = EMPTY_RENDERER_PASS_TIMINGS;

  /**
   * Sends the viewer's light, pointing and scaling a neutral noon rather than the game's warm one, so the asset shows
   * its own colours.
   *
   * @param lighting - The viewer's light, as its controls set it.
   */
  protected sendAssetLighting(lighting: IRenderLighting): void {
    this.sendLighting(toRendererLighting(lighting, NEUTRAL_RENDERER_LIGHTING));
  }

  protected onReport(report: IRendererReport): void {
    this.takeCost(report.frame, report);
  }

  protected onDetached(): void {
    this.takeCost(EMPTY_RENDER_FRAME_COST, EMPTY_RENDERER_PASS_TIMINGS);
  }

  private takeCost(cost: IRenderFrameCost, timings: IRendererPassTimings): void {
    runInAction(() => {
      this.frameCost = cost;
      this.timings = { isGpuTimed: timings.isGpuTimed, passes: timings.passes };
    });
  }
}
