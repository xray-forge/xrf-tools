import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { countSectorDraws, countSectorTriangles } from "@/core/level/lib/level-sector-views";

/** What a viewport costs, sampled rather than guessed. */
export interface ILevelStats {
  /** Mean frame time over the window, in milliseconds. */
  frameTime: number;
  /** Frames a second, derived from the mean rather than counted, so a short window still reports. */
  framesPerSecond: number;
  /** Sectors resident. */
  sectors: number;
  /** Draw calls the resident sectors cost: one per surface of each, and one per instanced mesh. */
  draws: number;
  /** Triangles drawn, an instanced mesh counted once for every place it stands. */
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
 * Measures what the resident sectors cost, without asking the renderer.
 *
 * @param sectors - What the loader currently holds.
 * @param frameTime - Mean frame time, from the viewport's own timer.
 * @returns What the viewport is spending.
 */
export function measureLevelStats(sectors: ReadonlyMap<number, ILoadedSector>, frameTime: number): ILevelStats {
  let draws: number = 0;
  let triangles: number = 0;
  let bytes: number = 0;

  for (const loaded of sectors.values()) {
    draws += countSectorDraws(loaded.views);
    triangles += countSectorTriangles(loaded.views);
    bytes += loaded.views.bufferLength;
  }

  return {
    bytes,
    draws,
    frameTime,
    framesPerSecond: frameTime > 0 ? 1000 / frameTime : 0,
    sectors: sectors.size,
    triangles,
  };
}
