import { Nullable } from "@xrf/types";

/** Whether a transport plays, as a media element's states name it. */
export type TAudioPlayback = "stopped" | "playing" | "paused" | "ended";

/** Where the sound a transport plays stands, in seconds. */
export interface IAudioTransportState {
  playback: TAudioPlayback;
  /** Seconds into the sound it plays or holds at. */
  position: number;
  /** Seconds the sound plays for. */
  duration: number;
  isLooped: boolean;
  /** Why playing failed, as the transport says it, or null. */
  failure: Nullable<string>;
}

/**
 * What a player plays one sound through: where it stands, its waveform, and what it can be asked. The player keeps no
 * idea of its own of what plays, so whatever plays the sound decides it.
 */
export interface IAudioTransport {
  state: IAudioTransportState;
  /** The loudest sample of each equal stretch of the sound, from 0 to 1, or null while it is unknown. */
  peaks: Nullable<Float32Array>;
  play(): void;
  pause(): void;
  /**
   * @param position - Seconds into the sound.
   */
  seek(position: number): void;
  setLooped(isLooped: boolean): void;
}
