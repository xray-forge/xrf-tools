import { Nullable } from "@xrf/types";

/**
 * Decides when queued records are due to be sent: after an interval, at once when a severe line hurries it but at
 * most once per interval, or at once when asked outright.
 */
export class LogFlushSchedule {
  private readonly interval: number;
  private readonly onDue: () => void;

  private timer: Nullable<ReturnType<typeof setTimeout>> = null;
  /** When a send was last hurried, on the monotonic clock. */
  private hurriedAt: number = -Infinity;
  private due: boolean = false;

  /**
   * @param interval - Milliseconds a line waits, and the least time between two hurried sends.
   * @param onDue - Called when the interval passes, the send then being due.
   */
  public constructor(interval: number, onDue: () => void) {
    this.interval = interval;
    this.onDue = onDue;
  }

  /** Whether what is queued should go as soon as nothing holds it. */
  public get isDue(): boolean {
    return this.due;
  }

  /** Makes a send due at once. */
  public markDue(): void {
    this.due = true;
  }

  /** Makes a send due at once unless one was hurried within the interval; then it waits for the interval. */
  public hurry(): void {
    const now: number = performance.now();

    if (now - this.hurriedAt >= this.interval) {
      this.hurriedAt = now;
      this.due = true;
    } else {
      this.wait();
    }
  }

  /** Starts the interval, unless it already runs or a send is due anyway. */
  public wait(): void {
    if (this.timer !== null || this.due) {
      return;
    }

    this.timer = setTimeout(() => {
      this.timer = null;
      this.due = true;
      this.onDue();
    }, this.interval);
  }

  /** Forgets the due send and the running interval, as a send starts or nothing is left to send. */
  public reset(): void {
    this.due = false;

    if (this.timer !== null) {
      clearTimeout(this.timer);
      this.timer = null;
    }
  }
}
