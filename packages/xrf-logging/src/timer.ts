/**
 * Elapsed-time reader for a log line.
 */
export class Timer {
  private readonly startedAt: number = performance.now();
  private lastAt: number = this.startedAt;

  /**
   * Reads the total, which laps do not reset.
   *
   * @returns Whole milliseconds since construction, matching the units rust commands put on the wire.
   */
  public elapsed(): number {
    return Math.round(performance.now() - this.startedAt);
  }

  /**
   * Reads the phase that just finished and starts the next one.
   *
   * @returns Whole milliseconds since the previous lap, or since construction for the first.
   */
  public lap(): number {
    const now: number = performance.now();
    const lap: number = Math.round(now - this.lastAt);

    this.lastAt = now;

    return lap;
  }
}
