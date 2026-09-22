import { Nullable } from "@xrf/types";

import { ArchiveLevelSndStaticSound } from "@/core/ipc/types/xrf-app";
import { formatNumber } from "@/lib/format/number";

/**
 * How loudly and how fast one planted sound plays.
 *
 * @param sound - Sound to describe.
 * @returns The volume with the pitch beside it.
 */
export function describePlayback(sound: ArchiveLevelSndStaticSound): string {
  return `volume ${formatNumber(sound.volume, 2)} · frequency ${formatNumber(sound.frequency, 2)}`;
}

/**
 * When of the day a planted sound is allowed to play.
 *
 * @param sound - Sound to describe.
 * @returns The hours it is allowed in, or null for a sound allowed at any hour.
 */
export function describeSchedule(sound: ArchiveLevelSndStaticSound): Nullable<string> {
  return sound.isScheduled ? `Between ${formatHour(sound.activeFrom)} and ${formatHour(sound.activeTo)}` : null;
}

/**
 * One end of a sound's window, which the engine stores and compares as a whole hour of the day.
 *
 * @param hour - Hour as stored.
 * @returns The hour as a clock reads it.
 */
function formatHour(hour: number): string {
  return `${hour.toString().padStart(2, "0")}:00`;
}
