import { WorldSoundAcousticsReport, WorldSoundReport, WorldSoundVoiceReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

/**
 * @param milliseconds - Milliseconds of the level's clock.
 * @returns Them as whole seconds, rounded up.
 */
export function formatLevelSoundSeconds(milliseconds: number): string {
  return `${Math.ceil(milliseconds / 1000)} s`;
}

/**
 * @param milliseconds - Milliseconds into a track.
 * @returns Them as `m:ss`, whole seconds rounded down.
 */
export function formatLevelSoundTrackTime(milliseconds: number): string {
  const seconds: number = Math.floor(Math.max(0, milliseconds) / 1000);

  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
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
 * @param share - A share from 0 to 1, null where it was not a number.
 * @returns It as a whole percentage.
 */
function formatShare(share: Nullable<number>): string {
  return `${Math.round((share ?? 0) * 100)}%`;
}

/**
 * @param voice - One sound the scene plays.
 * @returns How the level's walls and the listener's reverb shape it, and how it reaches the ears; empty for a 2D sound
 *   or one the audio has not taken.
 */
export function formatLevelSoundAcoustics(voice: WorldSoundVoiceReport): string {
  if (voice.occlusion === null || voice.distance === null) {
    return "";
  }

  const walls: string =
    voice.occlusion < 0.995
      ? `${formatShare(voice.occlusion)} unobstructed, walls pass ${(voice.transmission ?? [1, 1, 1]).map(formatShare).join("/")}`
      : "unobstructed";
  const reverb: string = voice.reverbSend ? `, reverb ${voice.reverbSend.toFixed(2)}` : "";

  return `${walls}${reverb}, ${voice.isBinaural ? "HRTF" : "panned"}`;
}

/**
 * @param acoustics - How the host's acoustics shape the scene, null until the audio has taken it.
 * @returns Whether they run and on what walls, or why the scene plays dry.
 */
export function formatLevelSoundAcousticsState(acoustics: Nullable<WorldSoundAcousticsReport>): string {
  if (!acoustics) {
    return "Not yet heard";
  }

  if (acoustics.failure) {
    return `Dry: ${acoustics.failure}`;
  }

  if (acoustics.geometryFailure) {
    return `Steam Audio, no walls: ${acoustics.geometryFailure}`;
  }

  if (acoustics.triangles === null) {
    return "Steam Audio, building the level's walls";
  }

  const reverb: string = acoustics.reverbTimes.map((time: Nullable<number>) => (time ?? 0).toFixed(2)).join("/");

  return `Steam Audio, ${acoustics.triangles.toLocaleString("en")} triangles, reverb ${reverb} s`;
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
