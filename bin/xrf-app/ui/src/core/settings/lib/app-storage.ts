import { formatCount } from "@xrf/format";
import { Nullable } from "@xrf/types";

/** A day in milliseconds. */
const DAY_MS: number = 24 * 60 * 60 * 1000;

/**
 * @param lastUsed - When any of a kind's files was last written or, for a cache, read; milliseconds since the epoch.
 * @param now - Now, in the same.
 * @returns How long ago, by the day, or nothing where the kind holds nothing.
 */
export function describeLastUse(lastUsed: Nullable<number>, now: number): string {
  if (lastUsed === null) {
    return "";
  }

  const days: number = Math.floor(Math.max(0, now - lastUsed) / DAY_MS);

  return days === 0 ? "used today" : days === 1 ? "used yesterday" : `used ${formatCount(days)} days ago`;
}
