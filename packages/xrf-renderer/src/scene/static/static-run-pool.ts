import { Nullable } from "@xrf/types";

import { DirtySpan } from "#/scene/dirty-span";
import { RangeAllocator } from "#/scene/static/range-allocator";
import { IStaticRuns } from "#/scene/static/static-runs";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";

/**
 * A pool of the static draw buffers handed out in runs: places, rows, impostors or clusters, what changed uploaded as
 * one span a buffer.
 */
export abstract class StaticRunPool implements IStaticRuns {
  /** Which pool of the buffers it is, which its first size and its limit are read by. */
  public readonly kind: EStaticPool;

  protected readonly buffers: StaticDrawBuffers;
  protected readonly runs: RangeAllocator = new RangeAllocator();
  /** The elements written since the buffers last went up. */
  protected readonly span: DirtySpan = new DirtySpan();
  protected currentVersion: number = 0;

  /**
   * @param buffers - What every static draw reads.
   * @param kind - Which pool of them it is.
   */
  protected constructor(buffers: StaticDrawBuffers, kind: EStaticPool) {
    this.buffers = buffers;
    this.kind = kind;
    this.runs.grow(buffers.capacity(kind));
  }

  public get capacity(): number {
    return this.runs.capacity;
  }

  public get used(): number {
    return this.runs.used;
  }

  /** Elements a cull has to look at: up to the end of the last run handed out. */
  public get extent(): number {
    return this.runs.extent;
  }

  /** Bumped whenever what it holds changes, or it grows, so a cull knows to run again. */
  public get version(): number {
    return this.currentVersion;
  }

  public fits(count: number): boolean {
    return this.runs.fits(count);
  }

  public allocate(count: number): Nullable<number> {
    return this.runs.allocate(count);
  }

  /**
   * @param capacity - What it holds from now on, more than it did: its buffers and its runs.
   */
  public grow(capacity: number): void {
    this.buffers.grow(this.kind, capacity);
    this.runs.grow(capacity);
    this.currentVersion += 1;
  }

  /**
   * @param start - Where a run handed out starts, holding nothing from now on.
   * @param count - Its length.
   */
  public abstract free(start: number, count: number): void;

  /** Marks what changed since the last upload to go up with the next use of the buffers. */
  public abstract flush(): void;
}
