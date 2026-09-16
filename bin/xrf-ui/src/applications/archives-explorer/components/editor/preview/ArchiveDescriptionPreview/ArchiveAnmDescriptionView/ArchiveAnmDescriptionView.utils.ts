import { ArchiveAnmDescription } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";
import { Nullable } from "@/lib/types/general";

/** How far a keyed reach may sit from the declared end before it is worth saying so, in seconds. */
const REACH_TOLERANCE: number = 0.05;

/**
 * How far the keys reach against the range the file declares, where the two disagree.
 *
 * @param description - Animation to weigh.
 * @returns A phrase for the disagreement, or null where the keys end where the range does.
 */
export function describeKeyedReach(description: ArchiveAnmDescription): Nullable<string> {
  const { keyedSeconds, durationSeconds } = description;

  if (
    keyedSeconds === null ||
    durationSeconds === null ||
    Math.abs(keyedSeconds - durationSeconds) <= REACH_TOLERANCE
  ) {
    return null;
  }

  const reach: string = formatNumber(keyedSeconds, 2);

  return keyedSeconds > durationSeconds
    ? `Reaching ${reach} s, past the end of the declared range`
    : `Reaching ${reach} s, short of the declared end`;
}
