import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { IRenderFrameCost } from "@/core/render/lib/frame/render-viewport";

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
}

export const EMPTY_LEVEL_STATS: ILevelStats = {
  bytes: 0,
  drawTime: 0,
  draws: 0,
  frameTime: 0,
  framesPerSecond: 0,
  sceneTime: 0,
  sectors: 0,
  triangles: 0,
  worstDrawTime: 0,
  worstFrameTime: 0,
};

/**
 * Measures what the viewport is holding, against what its last frame cost.
 *
 * @param sectors - What the loader currently holds.
 * @param frame - What the viewport's renderer counted for the frame just drawn.
 * @param sceneTime - What the scene has been paying to take one arriving sector.
 * @returns What the viewport is spending.
 */
export function measureLevelStats(
  sectors: ReadonlyMap<number, ILoadedSector>,
  frame: IRenderFrameCost,
  sceneTime: number = 0
): ILevelStats {
  let bytes: number = 0;

  for (const loaded of sectors.values()) {
    bytes += loaded.views.bufferLength;
  }

  return {
    bytes,
    drawTime: frame.drawTime,
    draws: frame.draws,
    frameTime: frame.frameTime,
    framesPerSecond: frame.framesPerSecond,
    sceneTime,
    sectors: sectors.size,
    triangles: frame.triangles,
    worstDrawTime: frame.worstDrawTime,
    worstFrameTime: frame.worstFrameTime,
  };
}
