import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { Nullable } from "@/lib/types/general";

/** What a viewport costs, sampled rather than guessed. */
export interface ILevelStats {
  /** Mean frame time over the window, in milliseconds. */
  frameTime: number;
  /** Frames a second, derived from the mean rather than counted, so a short window still reports. */
  framesPerSecond: number;
  /** Sectors resident. */
  sectors: number;
  /** Draw calls the resident sectors cost, which is one per surface of each. */
  draws: number;
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

/** Frames averaged over. Short enough to react to a camera entering a dense sector, long enough not to flicker. */
const WINDOW: number = 30;

/**
 * A rolling mean of frame times.
 */
export class LevelFrameTimer {
  private readonly samples: Array<number> = [];

  /** Null rather than zero: a render loop whose clock starts at zero would otherwise lose its first frame. */
  private last: Nullable<number> = null;

  /**
   * Records one frame.
   *
   * @param now - Timestamp of this frame, as the render loop received it.
   */
  public sample(now: number): void {
    if (this.last !== null) {
      this.samples.push(now - this.last);

      if (this.samples.length > WINDOW) {
        this.samples.shift();
      }
    }

    this.last = now;
  }

  /**
   * @returns Mean frame time over the window, or zero before two frames have been seen.
   */
  public get frameTime(): number {
    if (!this.samples.length) {
      return 0;
    }

    return this.samples.reduce((total: number, sample: number) => total + sample, 0) / this.samples.length;
  }

  /** Forgets the window, for a viewport that was hidden or has swapped levels. */
  public reset(): void {
    this.samples.length = 0;
    this.last = null;
  }
}

/**
 * Measures what the resident sectors cost, without asking the renderer.
 *
 * @param sectors - What the loader currently holds.
 * @param frameTime - Mean frame time, from the loop's own timer.
 * @returns What the viewport is spending.
 */
export function measureLevelStats(sectors: ReadonlyMap<number, ILoadedSector>, frameTime: number): ILevelStats {
  let draws: number = 0;
  let triangles: number = 0;
  let bytes: number = 0;

  for (const loaded of sectors.values()) {
    draws += loaded.views.sections.length;
    bytes += loaded.views.bufferLength;

    for (const section of loaded.views.sections) {
      triangles += section.triangleCount;
    }
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
