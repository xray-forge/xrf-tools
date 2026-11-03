import { WorldSoundReport, WorldSoundVoiceReport } from "@xrf/ipc/types/xrf-world";

/**
 * @param milliseconds - Milliseconds of the level's clock.
 * @returns Them as whole seconds, rounded up.
 */
export function formatLevelSoundSeconds(milliseconds: number): string {
  return `${Math.ceil(milliseconds / 1000)} s`;
}

/**
 * @param milliseconds - Milliseconds since midnight.
 * @returns The time of day they name, `hh:mm`.
 */
export function formatLevelSoundTimeOfDay(milliseconds: number): string {
  const minutes: number = Math.floor(milliseconds / 60_000);

  return `${String(Math.floor(minutes / 60) % 24).padStart(2, "0")}:${String(minutes % 60).padStart(2, "0")}`;
}

/**
 * @param voice - One sound the scene plays.
 * @returns How it is heard: its gain in decibels, or why it is not.
 */
export function formatLevelSoundGain(voice: WorldSoundVoiceReport): string {
  if (voice.isLoading) {
    return "Reading";
  }

  if (voice.gain === null || voice.gain <= 0) {
    return "Silent";
  }

  return `${(20 * Math.log10(voice.gain)).toFixed(1)} dB`;
}

/**
 * @param report - What the scene plays.
 * @returns Its voices, the real ones first, each group loudest first.
 */
export function sortLevelSoundVoices(report: WorldSoundReport): Array<WorldSoundVoiceReport> {
  return [...report.voices].sort(
    (a: WorldSoundVoiceReport, b: WorldSoundVoiceReport) =>
      Number(b.isReal) - Number(a.isReal) || (b.gain ?? 0) - (a.gain ?? 0)
  );
}
