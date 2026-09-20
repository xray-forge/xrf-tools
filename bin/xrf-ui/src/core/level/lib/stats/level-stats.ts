import { ILoadedSector } from "@/core/level/lib/sector/level-sector-set";
import { IRenderFrameCost } from "@/core/render/lib/render-viewport";

/**
 * What a viewport is holding, against what a frame of it costs.
 */
export interface ILevelStats {
  /** Mean frame time over the window, in milliseconds. */
  frameTime: number;
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
}

export const EMPTY_LEVEL_STATS: ILevelStats = {
  bytes: 0,
  draws: 0,
  frameTime: 0,
  framesPerSecond: 0,
  sectors: 0,
  triangles: 0,
};

/**
 * Measures what the viewport is holding, against what its last frame cost.
 *
 * @param sectors - What the loader currently holds.
 * @param frame - What the viewport's renderer counted for the frame just drawn.
 * @returns What the viewport is spending.
 */
export function measureLevelStats(sectors: ReadonlyMap<number, ILoadedSector>, frame: IRenderFrameCost): ILevelStats {
  let bytes: number = 0;

  for (const loaded of sectors.values()) {
    bytes += loaded.views.bufferLength;
  }

  return {
    bytes,
    draws: frame.draws,
    frameTime: frame.frameTime,
    framesPerSecond: frame.framesPerSecond,
    sectors: sectors.size,
    triangles: frame.triangles,
  };
}
