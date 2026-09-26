import {
  EMPTY_RENDER_FRAME_COST,
  EMPTY_RENDERER_LIGHTS_REPORT,
  EMPTY_RENDERER_STATIC_DRAW_REPORT,
  IRendererLightsReport,
  IRendererStaticDrawReport,
  IRenderFrameCost,
} from "@xrf/renderer";

/**
 * What a viewport is holding, against what a frame of it costs.
 */
export interface ILevelStats extends IRenderFrameCost {
  /** Sectors resident. */
  sectors: number;
  /** Bytes of geometry held, which is what a residency budget is really spending. */
  bytes: number;
  /** Mean milliseconds one arriving sector costs to put into the scene. */
  sceneTime: number;
  /** How full the static draws' pools are, how often one fell back to drawing plainly, and what occlusion removed. */
  staticDraws: IRendererStaticDrawReport;
  /** What the local lights came to: how many stood in view and were shadowed, the atlas, and full clusters. */
  lights: IRendererLightsReport;
}

export const EMPTY_LEVEL_STATS: ILevelStats = {
  ...EMPTY_RENDER_FRAME_COST,
  bytes: 0,
  lights: EMPTY_RENDERER_LIGHTS_REPORT,
  sceneTime: 0,
  sectors: 0,
  staticDraws: EMPTY_RENDERER_STATIC_DRAW_REPORT,
};

/** What is held, which only whoever holds it can say. */
export interface ILevelHeld {
  /** Sectors resident. */
  sectors: number;
  /** Bytes of geometry they came to. */
  bytes: number;
}

/**
 * Measures what the viewport is holding, against what its last frame cost.
 *
 * @param held - How many sectors are resident and what they came to.
 * @param frame - What the viewport's renderer counted for the frame just drawn.
 * @param sceneTime - What the scene has been paying to take one arriving sector.
 * @param staticDraws - What the renderer said of its static draws.
 * @param lights - What it said of its local lights.
 * @returns What the viewport is spending.
 */
export function measureLevelStats(
  held: ILevelHeld,
  frame: IRenderFrameCost,
  sceneTime: number,
  staticDraws: IRendererStaticDrawReport,
  lights: IRendererLightsReport
): ILevelStats {
  return { ...frame, bytes: held.bytes, lights, sceneTime, sectors: held.sectors, staticDraws };
}
