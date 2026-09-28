import { IStaticRuns } from "#/scene/static/static-runs";
import { EStaticPool } from "#/uniforms/static-pool";

/**
 * A pool of the static draw buffers handed out in runs: places, rows, impostors or clusters, each uploaded as one span.
 */
export interface IStaticRunPool extends IStaticRuns {
  /** Which pool of the buffers it is, which its first size and its limit are read by. */
  readonly kind: EStaticPool;
  /** Bumped whenever what it holds changes, so a cull knows to run again. */
  readonly version: number;
  /**
   * @param capacity - What it holds from now on, more than it did: its buffers and its runs.
   */
  grow(capacity: number): void;
  /**
   * @param start - Where a run handed out starts, holding nothing from now on.
   * @param count - Its length.
   */
  free(start: number, count: number): void;
  /** Marks what changed since the last upload to go up with the next use of the buffers. */
  flush(): void;
}
