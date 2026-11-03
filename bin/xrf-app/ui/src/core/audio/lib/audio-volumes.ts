import { AudioVolumes } from "@xrf/ipc/types/xrf-app-audio";
import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";

/** A category the settings give a volume of its own, the master's included. */
export type TAudioVolume = keyof AudioVolumes;

/** The volumes the settings offer, in the order they list them. */
export const AUDIO_VOLUMES: ReadonlyArray<TAudioVolume> = ["master", "music", "ambience", "world", "previews"];

export const AUDIO_VOLUME_LABELS: Readonly<Record<TAudioVolume, string>> = {
  master: "Master",
  music: "Music",
  ambience: "Ambience",
  world: "World",
  previews: "Previews",
};

export const AUDIO_VOLUME_DESCRIPTIONS: Readonly<Record<TAudioVolume, string>> = {
  master: "Everything the application plays.",
  music: "A level's music tracks.",
  ambience: "Static sounds, the weather's ambient channels, rain, thunder and ambient effects.",
  world: "Sounds at the world's objects: anomalies and campfires.",
  previews: "Sounds a tool previews, such as an archived sound.",
};

/**
 * One volume as a slider shows it: in `[0, 1]`, a value the wire could not carry as a number at full volume.
 *
 * @param volume - The volume as the backend sent it.
 * @returns The volume in range.
 */
export function readAudioVolume(volume: Nullable<number> | undefined): number {
  return typeof volume === "number" && Number.isFinite(volume) ? saturate(volume) : 1;
}
