import { Nullable } from "@xrf/types";

/**
 * Runs of a buffer's elements handed out, which a growth of the buffer is sized by.
 */
export interface IStaticRuns {
  /** Elements the buffer holds. */
  readonly capacity: number;
  /** Elements handed out and not freed. */
  readonly used: number;
  /** Where the last run handed out ends: nothing at or past it is in use. */
  readonly extent: number;
  /** @returns Whether a free run is `count` long, without taking it. */
  fits(count: number): boolean;
  /** @returns Where a run of `count` starts, or null where no free run is that long. */
  allocate(count: number): Nullable<number>;
}
