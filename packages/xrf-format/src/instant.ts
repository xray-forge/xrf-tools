import { format, isSameDay } from "date-fns";

/**
 * Formats a wall-clock moment, spelling its date only when it is not the reader's own day.
 *
 * A bare clock time is the most readable form and the right one almost always, because what is being timestamped is
 * usually minutes old. It becomes a lie the moment it is not: `23:58` on a window that has been open since yesterday
 * reads as two minutes ago.
 *
 * @param at - Milliseconds since the epoch.
 * @param now - Moment to read it against, passed by the caller so this can be tested.
 * @returns The clock time, prefixed with the date when the two fall on different days.
 */
export function formatInstant(at: number, now: number = Date.now()): string {
  return isSameDay(at, now) ? format(at, "HH:mm:ss") : format(at, "d MMM HH:mm:ss");
}
