import { EMPTY_RENDERER_STATIC_DRAW_REPORT, IRendererStaticDrawReport, IRenderFrameCost } from "@xrf/renderer";

/**
 * What a viewport is holding, against what a frame of it costs.
 */
export interface ILevelStats {
  /** Mean frame time over the window, in milliseconds. */
  frameTime: number;
  /** The longest frame of the window, which is the stutter rather than the average of it. */
  worstFrameTime: number;
  /** Mean of what drawing cost, which is where uploads and shader compiles land. */
  drawTime: number;
  /** The longest draw of the window, which says whether a spike was inside `render` or outside it. */
  worstDrawTime: number;
  /** Frames a second, derived from the mean rather than counted, so a short window still reports. */
  framesPerSecond: number;
  /** Sectors resident. */
  sectors: number;
  /** Draw calls the last frame issued, which is what survived culling rather than what is held. */
  draws: number;
  /** Triangles the last frame drew, instanced geometry counted once for every place it stood. */
  triangles: number;
  /** Bytes of geometry held, which is what a residency budget is really spending. */
  bytes: number;
  /** Mean milliseconds one arriving sector costs to put into the scene. */
  sceneTime: number;
  /** Width of the buffer the frame was drawn into, in device pixels. */
  drawnWidth: number;
  /** Height of the same buffer. Everything above is paid for over these two numbers. */
  drawnHeight: number;
  /** How full the static draws' pools are, how often one fell back to drawing plainly, and what occlusion removed. */
  staticDraws: IRendererStaticDrawReport;
}

export const EMPTY_LEVEL_STATS: ILevelStats = {
  bytes: 0,
  drawnHeight: 0,
  drawnWidth: 0,
  drawTime: 0,
  draws: 0,
  frameTime: 0,
  framesPerSecond: 0,
  sceneTime: 0,
  sectors: 0,
  staticDraws: EMPTY_RENDERER_STATIC_DRAW_REPORT,
  triangles: 0,
  worstDrawTime: 0,
  worstFrameTime: 0,
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
 * @returns What the viewport is spending.
 */
export function measureLevelStats(
  held: ILevelHeld,
  frame: IRenderFrameCost,
  sceneTime: number = 0,
  staticDraws: IRendererStaticDrawReport = EMPTY_RENDERER_STATIC_DRAW_REPORT
): ILevelStats {
  return {
    bytes: held.bytes,
    drawnHeight: frame.drawnHeight,
    drawnWidth: frame.drawnWidth,
    drawTime: frame.drawTime,
    draws: frame.draws,
    frameTime: frame.frameTime,
    framesPerSecond: frame.framesPerSecond,
    sceneTime,
    sectors: held.sectors,
    staticDraws,
    triangles: frame.triangles,
    worstDrawTime: frame.worstDrawTime,
    worstFrameTime: frame.worstFrameTime,
  };
}
