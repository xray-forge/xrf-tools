import { Maybe, Nullable } from "@xrf/types";

/** A run of free elements. */
interface IFreeRange {
  start: number;
  count: number;
}

/**
 * Hands out runs of a buffer's elements, first fit, joining a released run with the free ones beside it.
 */
export class RangeAllocator {
  private readonly free: Array<IFreeRange> = [];
  private currentCapacity: number = 0;

  /** Elements the buffer holds. */
  public get capacity(): number {
    return this.currentCapacity;
  }

  /** Where the last run handed out ends: nothing at or past it is in use. */
  public get extent(): number {
    const last: Maybe<IFreeRange> = this.free[this.free.length - 1];

    return last && last.start + last.count === this.currentCapacity ? last.start : this.currentCapacity;
  }

  /** Elements handed out and not released. */
  public get used(): number {
    return this.free.reduce((total: number, range: IFreeRange) => total - range.count, this.currentCapacity);
  }

  /**
   * @param count - Elements wanted.
   * @returns Where the run starts, or null where no free run is that long.
   */
  public allocate(count: number): Nullable<number> {
    const index: number = this.free.findIndex((range: IFreeRange) => range.count >= count);

    if (index < 0) {
      return null;
    }

    const range: IFreeRange = this.free[index];
    const start: number = range.start;

    range.start += count;
    range.count -= count;

    if (!range.count) {
      this.free.splice(index, 1);
    }

    return start;
  }

  /**
   * @param start - Where a run handed out starts.
   * @param count - Its length.
   */
  public release(start: number, count: number): void {
    if (!count) {
      return;
    }

    let index: number = this.free.findIndex((range: IFreeRange) => range.start > start);

    if (index < 0) {
      index = this.free.length;
    }

    this.free.splice(index, 0, { count, start });
    this.join(index);

    if (index > 0) {
      this.join(index - 1);
    }
  }

  /**
   * @param capacity - What the buffer holds now, more than before; the new elements are free.
   */
  public grow(capacity: number): void {
    this.release(this.currentCapacity, capacity - this.currentCapacity);
    this.currentCapacity = capacity;
  }

  /** Joins a free run with the one after it, where they touch. */
  private join(index: number): void {
    const range: IFreeRange = this.free[index];
    const next: IFreeRange = this.free[index + 1];

    if (next && range.start + range.count === next.start) {
      range.count += next.count;
      this.free.splice(index + 1, 1);
    }
  }
}
