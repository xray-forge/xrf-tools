import { WorldSoundAcousticsReport, WorldSoundReport, WorldSoundVoiceReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

/**
 * Level clock milliseconds as whole seconds, rounded up.
 */
export function formatLevelSoundSeconds(milliseconds: number): string {
  return `${Math.ceil(milliseconds / 1000)} s`;
}

/**
 * A track position in milliseconds as `m:ss`, rounded down.
 */
export function formatLevelSoundTrackTime(milliseconds: number): string {
  const seconds: number = Math.floor(Math.max(0, milliseconds) / 1000);

  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, "0")}`;
}

/**
 * Milliseconds since midnight as `hh:mm`.
 */
export function formatLevelSoundTimeOfDay(milliseconds: number): string {
  const minutes: number = Math.floor(milliseconds / 60_000);

  return `${String(Math.floor(minutes / 60) % 24).padStart(2, "0")}:${String(minutes % 60).padStart(2, "0")}`;
}

/**
 * The mixer's load as a percentage of the audio period.
 *
 * @param load - Share of each period the mixer spends, from 0 to 1.
 */
export function formatLevelSoundMixerLoad(load: number): string {
  return `${(load * 100).toFixed(1)}% of the period`;
}

/**
 * A voice's kind, distance or 2D, gain, restarts, and for a real one its acoustics.
 */
export function formatLevelSoundVoice(voice: WorldSoundVoiceReport): string {
  const distance: string = voice.distance === null ? "2D" : `${voice.distance.toFixed(1)} m`;
  const gain: string = voice.isReal ? formatLevelSoundGain(voice) : `simulated at ${formatLevelSoundGain(voice)}`;
  const restarts: string = voice.restarts > 0 ? `, started over ${voice.restarts}×` : "";
  const acoustics: string = voice.isReal ? formatLevelSoundAcoustics(voice) : "";

  return `${voice.kind}, ${distance}, ${gain}${restarts}${acoustics ? `; ${acoustics}` : ""}`;
}

/**
 * A voice's gain in decibels, or why it is not heard.
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
 * A 0 to 1 share as a whole percentage; null, where it was not a number, reads as zero.
 */
function formatShare(share: Nullable<number>): string {
  return `${Math.round((share ?? 0) * 100)}%`;
}

/**
 * How walls, reverb and panning shape a voice; empty for a 2D sound or one the audio has not taken.
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
 * Whether the acoustics run and on what walls, or why the scene plays dry.
 *
 * @param acoustics - Null until the audio has taken the scene.
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
 * The scene's voices, real ones first, each group loudest first.
 */
export function sortLevelSoundVoices(report: WorldSoundReport): Array<WorldSoundVoiceReport> {
  return [...report.voices].sort(
    (a: WorldSoundVoiceReport, b: WorldSoundVoiceReport) =>
      Number(b.isReal) - Number(a.isReal) || (b.gain ?? 0) - (a.gain ?? 0)
  );
}
