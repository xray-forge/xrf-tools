import { Nullable } from "@xrf/types";

import { IStaticRuns } from "#/scene/static/static-runs";

/**
 * The room a growing static buffer leaves beyond what it holds and what is known to be coming, for what streams in
 * later: every growth copies and uploads all of it again, a hitch.
 */
export const STATIC_HEADROOM: number = 1.25;

/**
 * @param used - Elements the buffer holds in use.
 * @param wanted - Elements about to be placed beside them: the one asking, and what the queue is known to bring.
 * @param capacity - What it holds now.
 * @param initial - What it holds at the least.
 * @param limit - What it may hold at the most.
 * @returns What it grows to: the wanted elements beside the used ones with headroom, at least its initial size, more
 *   than it was, and never past its limit.
 */
export function toGrownCapacity(
  used: number,
  wanted: number,
  capacity: number,
  initial: number,
  limit: number
): number {
  return Math.min(limit, Math.max(initial, capacity + 1, Math.ceil((used + wanted) * STATIC_HEADROOM)));
}

/**
 * What a buffer grows to for a run to fit, in one growth: past what it holds and what the queue brings, and past the
 * end of its last run by the whole run, since freed room may lie in runs too short for it.
 *
 * @param runs - The runs handed out of the buffer.
 * @param count - Elements the run takes.
 * @param initial - What the buffer holds at the least.
 * @param limit - What it may hold at the most.
 * @param toWanted - What the queue is known to bring besides, asked only where the buffer has to grow.
 * @returns The capacity: what it holds already where the run fits, or null where the limit stops it fitting.
 */
export function toFittedCapacity(
  runs: IStaticRuns,
  count: number,
  initial: number,
  limit: number,
  toWanted: () => number = () => 0
): Nullable<number> {
  if (runs.fits(count)) {
    return runs.capacity;
  }

  const capacity: number = Math.max(
    toGrownCapacity(runs.used, count + toWanted(), runs.capacity, initial, limit),
    toGrownCapacity(runs.extent, count, runs.capacity, initial, limit)
  );

  // What growing adds joins the free run the last run leaves at the end.
  return runs.extent + count <= capacity ? capacity : null;
}

/**
 * @param runs - The runs handed out of a buffer.
 * @param count - Elements a run takes.
 * @param initial - What the buffer holds at the least.
 * @param limit - What it may hold at the most.
 * @param grow - Grows the buffer and its runs to the capacity given.
 * @param toWanted - What the queue is known to bring besides, asked only where the buffer has to grow.
 * @returns Where the run starts, the buffer grown once to fit it (`toFittedCapacity`); null where the limit stops it,
 *   nothing grown.
 */
export function allocateGrowing(
  runs: IStaticRuns,
  count: number,
  initial: number,
  limit: number,
  grow: (capacity: number) => void,
  toWanted: () => number = () => 0
): Nullable<number> {
  const capacity: Nullable<number> = toFittedCapacity(runs, count, initial, limit, toWanted);

  if (capacity === null) {
    return null;
  }

  if (capacity > runs.capacity) {
    grow(capacity);
  }

  return runs.allocate(count);
}
