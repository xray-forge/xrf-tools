// Auto-generated rust bindings. Do not edit it manually.

import { AudioOutputChoice } from "#/types/xrf-audio";

/**
 * How the application plays sound, kept by the backend between runs so it holds before any page loads; stored
 * settings that no longer read whole are left for the defaults.
 */
export type AudioSettings = {
  /** The device sound plays on. */
  output: AudioOutputChoice;
  volumes: AudioVolumes;
  /** Whether sound falls silent while no window of the application has the focus. */
  isMutedInBackground: boolean;
};

/**
 * How loud each category plays under the master, each from 0 (silent) to 1 (as recorded), linear as a media element's
 * volume is.
 */
export type AudioVolumes = {
  master: number | null;
  /** A level's music tracks. */
  music: number | null;
  /** Static sounds, the weather's ambient channels, rain, thunder and ambient effects. */
  ambience: number | null;
  /** Sounds at the world's objects: anomalies and campfires. */
  world: number | null;
  /** Sounds a tool previews, such as an archived sound. */
  previews: number | null;
};
