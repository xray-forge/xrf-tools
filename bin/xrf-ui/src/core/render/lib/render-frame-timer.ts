import { Nullable } from "@/lib/types/general";

/** Frames averaged over. Short enough to react to a camera entering dense geometry, long enough not to flicker. */
const WINDOW: number = 30;

/**
 * A rolling mean of frame times.
 */
export class RenderFrameTimer {
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

  /**
   * @returns Frames a second, derived from the mean rather than counted, so a short window still reports.
   */
  public get framesPerSecond(): number {
    const frameTime: number = this.frameTime;

    return frameTime > 0 ? 1000 / frameTime : 0;
  }

  /** Forgets the window, for a viewport that was hidden or has swapped subjects. */
  public reset(): void {
    this.samples.length = 0;
    this.last = null;
  }
}
