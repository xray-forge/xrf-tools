import { Nullable } from "@xrf/types";

import { TFrameRateLimit, toFrameInterval } from "#/frame/render-frame-limit";

/**
 * How much earlier than due a frame may land and still be drawn: a display's wakes wander by a fraction of a
 * millisecond, and one landing just short would otherwise wait a whole wake more.
 */
const FRAME_SLACK: number = 1;

/**
 * Paces frames to a rate on a display that wakes at its own. Each frame is due an interval after the one before it was
 * due, not after it was drawn, so the wakes it waits even out: a 60 limit on a 160 Hz display waits two wakes and
 * three by turns, and draws sixty a second rather than the fifty-three whole wakes would allow.
 */
export class RenderFrameLimiter {
  /** When the next frame is due, or null before the first. */
  private due: Nullable<number> = null;

  /**
   * @param now - The wake's own timestamp, in milliseconds.
   * @param limit - The rate asked for.
   * @returns Whether a frame is drawn at this wake.
   */
  public take(now: number, limit: TFrameRateLimit): boolean {
    const interval: number = toFrameInterval(limit);

    // Unlimited, or a whole interval behind: nothing to catch up with, the next frame is due from this one.
    if (interval === 0 || this.due === null || now - this.due >= interval) {
      this.due = now + interval;

      return true;
    }

    if (now + FRAME_SLACK < this.due) {
      return false;
    }

    this.due += interval;

    return true;
  }

  /** Forgets when the next frame was due, as a view shown again starts afresh. */
  public reset(): void {
    this.due = null;
  }
}
