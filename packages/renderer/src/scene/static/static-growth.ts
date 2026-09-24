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
