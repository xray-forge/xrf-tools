import { toMean, toWorst } from "@xrf/math";
import { Nullable } from "@xrf/types";

/** Frames averaged over. Short enough to react to a camera entering dense geometry, long enough not to flicker. */
const WINDOW: number = 30;

/**
 * A rolling mean of frame times.
 */
export class RenderFrameTimer {
  private readonly samples: Array<number> = [];

  /** What `render` itself took, kept apart because it is the half a mean cannot tell you the cause of. */
  private readonly draws: Array<number> = [];

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
   * Records what drawing one frame cost, which is where uploads and shader compiles land.
   *
   * @param elapsed - Milliseconds drawing the frame took.
   */
  public sampleDraw(elapsed: number): void {
    this.draws.push(elapsed);

    if (this.draws.length > WINDOW) {
      this.draws.shift();
    }
  }

  /**
   * @returns Mean frame time over the window, or zero before two frames have been seen.
   */
  public get frameTime(): number {
    return toMean(this.samples);
  }

  /**
   * @returns The longest frame of the window.
   *
   * A stutter is a worst case, and a mean over thirty frames is exactly the statistic that hides one: sixty
   * milliseconds once a second reads as four milliseconds added to every frame.
   */
  public get worstFrameTime(): number {
    return toWorst(this.samples);
  }

  /**
   * @returns Mean of what drawing cost over the window.
   */
  public get drawTime(): number {
    return toMean(this.draws);
  }

  /**
   * @returns The longest draw of the window, which says whether a spike was inside `render` or outside it.
   */
  public get worstDrawTime(): number {
    return toWorst(this.draws);
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
    this.draws.length = 0;
    this.last = null;
  }
}
