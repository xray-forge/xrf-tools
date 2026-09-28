/**
 * Two of a thing a frame writes one of while reading the other, which the next frame swaps.
 */
export class PingPong<T> {
  public readonly both: readonly [T, T];

  private written: 0 | 1 = 0;

  /**
   * @param create - Makes each of the two, given which it is.
   */
  public constructor(create: (index: 0 | 1) => T) {
    this.both = [create(0), create(1)];
  }

  /** The one this frame writes. */
  public get current(): T {
    return this.both[this.written];
  }

  /** The one the frame before wrote, which this frame reads. */
  public get previous(): T {
    return this.both[this.written === 0 ? 1 : 0];
  }

  /** Makes this frame's the one the next frame reads. */
  public swap(): void {
    this.written = this.written === 0 ? 1 : 0;
  }
}
