// Auto-generated rust bindings. Do not edit it manually.

/** An output device the system offers. */
export type AudioDeviceDescription = {
  /** The device's id, stable across runs, which an output choice of one device names it by. */
  id: string;
  name: string;
  /** Whether the system plays on it by default now. */
  isDefault: boolean;
};

/** Every `kind` the `AudioOutputChoice` union is told apart by, so a switch or a comparison names one. */
export enum EAudioOutputChoice {
  /** The system's default device, followed as the system changes it. */
  SYSTEM_DEFAULT = "systemDefault",
  /** One device by its id; the system default plays while it is missing. */
  DEVICE = "device",
}

/** Which device the audio plays on. */
export type AudioOutputChoice =
  /** The system's default device, followed as the system changes it. */
  | { kind: "systemDefault" }
  /** One device by its id; the system default plays while it is missing. */
  | { kind: "device"; id: string };

/** What the output plays on now. */
export type AudioOutputStatus = {
  /** The device playing, or `None` while none could be opened. */
  deviceName: string | null;
  sampleRate: number;
  channels: number;
  /** Whether the system default plays because the chosen device is missing. */
  isFallback: boolean;
  /** Why no device plays, while none does. */
  failure: string | null;
};

/** One transport an [`crate::AudioServer`] opened, which its caller plays, steers and closes by it. */
export type AudioTransportId = number;

/** Whether a transport plays, as a media element's states name it. */
export enum EAudioTransportPlayback {
  /** At the start, or put back there. */
  STOPPED = "stopped",
  PLAYING = "playing",
  /** Held where it was paused or sought to. */
  PAUSED = "paused",
  /** Played to its end without looping; playing again starts over. */
  ENDED = "ended",
}

/** Every `EAudioTransportPlayback` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type AudioTransportPlayback = `${EAudioTransportPlayback}`;

/** Where a transport is, for a player to show. */
export type AudioTransportState = {
  playback: AudioTransportPlayback;
  /** How far into the sound it plays or holds. */
  position: number;
  duration: number;
  isLooped: boolean;
  /** Why playing failed: the stream stopped decoding, or the audio thread refused the voice. */
  failure: string | null;
};

/**
 * A sound's waveform: the loudest sample of each equal stretch of its frames, over every channel, and how long it
 * plays.
 */
export type SoundPeaks = {
  /** Each stretch's largest absolute sample, from 0 to 1. */
  peaks: Array<number | null>;
  duration: number;
};
