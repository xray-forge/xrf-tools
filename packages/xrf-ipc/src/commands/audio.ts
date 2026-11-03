// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { AudioSettings } from "#/types/xrf-app-audio";
import {
  AudioDeviceDescription,
  AudioOutputStatus,
  AudioTransportId,
  AudioTransportState,
  SoundPeaks,
} from "#/types/xrf-audio";

/** The wire name of every command of the `audio` plugin. */
export enum EAudioCommand {
  CLOSE_PREVIEW = "plugin:audio|close_preview",
  DESCRIBE_OUTPUT = "plugin:audio|describe_output",
  DESCRIBE_SETTINGS = "plugin:audio|describe_settings",
  LIST_DEVICES = "plugin:audio|list_devices",
  PAUSE_PREVIEW = "plugin:audio|pause_preview",
  PLAY_PREVIEW = "plugin:audio|play_preview",
  READ_PREVIEW_PEAKS = "plugin:audio|read_preview_peaks",
  SEEK_PREVIEW = "plugin:audio|seek_preview",
  SET_PREVIEW_LOOPED = "plugin:audio|set_preview_looped",
  SET_SETTINGS = "plugin:audio|set_settings",
  STOP_PREVIEW = "plugin:audio|stop_preview",
  WATCH_PREVIEW = "plugin:audio|watch_preview",
}

/** Commands */
export const audioCommands = {
  /** Stop and forget the preview, as its player goes; one already closed is left alone. */
  closePreview: (transport: AudioTransportId) => __TAURI_INVOKE<void>(EAudioCommand.CLOSE_PREVIEW, { transport }),
  /** Say which device sound plays on now, or why none does. */
  describeOutput: () => __TAURI_INVOKE<AudioOutputStatus>(EAudioCommand.DESCRIBE_OUTPUT),
  /** Say how the application plays sound: its device, volumes and background mute. */
  describeSettings: () => __TAURI_INVOKE<AudioSettings>(EAudioCommand.DESCRIBE_SETTINGS),
  /** Name the output devices the system offers now, the default one marked, for the settings to offer. */
  listDevices: () => __TAURI_INVOKE<Array<AudioDeviceDescription>>(EAudioCommand.LIST_DEVICES),
  /** Hold the preview where it plays now. */
  pausePreview: (transport: AudioTransportId) => __TAURI_INVOKE<null>(EAudioCommand.PAUSE_PREVIEW, { transport }),
  /** Play the preview from where it holds, or from the start once it has ended. */
  playPreview: (transport: AudioTransportId) => __TAURI_INVOKE<null>(EAudioCommand.PLAY_PREVIEW, { transport }),
  /** The preview's waveform: the loudest sample of each of `count` equal stretches of its sound, decoded on the pool. */
  readPreviewPeaks: (transport: AudioTransportId, count: number) =>
    __TAURI_INVOKE<SoundPeaks>(EAudioCommand.READ_PREVIEW_PEAKS, { transport, count }),
  /**
   * Move the preview to `position` milliseconds in, clamped to the sound: a playing preview plays on from there, any
   * other holds there paused.
   */
  seekPreview: (transport: AudioTransportId, position: number) =>
    __TAURI_INVOKE<null>(EAudioCommand.SEEK_PREVIEW, { transport, position }),
  /** Whether the preview starts over at its end. */
  setPreviewLooped: (transport: AudioTransportId, isLooped: boolean) =>
    __TAURI_INVOKE<null>(EAudioCommand.SET_PREVIEW_LOOPED, { transport, isLooped }),
  /** Play with `settings` from now on and keep them for the next run, answering them as kept, every volume in range. */
  setSettings: (settings: AudioSettings) => __TAURI_INVOKE<AudioSettings>(EAudioCommand.SET_SETTINGS, { settings }),
  /** Put the preview back at its start, silent. */
  stopPreview: (transport: AudioTransportId) => __TAURI_INVOKE<null>(EAudioCommand.STOP_PREVIEW, { transport }),
  /** Send the preview's state to `states` now and whenever it changes, its position included, until it closes. */
  watchPreview: (transport: AudioTransportId, states: Channel<AudioTransportState>) =>
    __TAURI_INVOKE<null>(EAudioCommand.WATCH_PREVIEW, { transport, states }),
};

/** What each command of the `audio` plugin takes and resolves to, keyed by its wire name. */
export interface IAudioCommandSignatures {
  [EAudioCommand.CLOSE_PREVIEW]: IIpcCommandSignature<typeof audioCommands.closePreview, ["transport"]>;
  [EAudioCommand.DESCRIBE_OUTPUT]: IIpcCommandSignature<typeof audioCommands.describeOutput, []>;
  [EAudioCommand.DESCRIBE_SETTINGS]: IIpcCommandSignature<typeof audioCommands.describeSettings, []>;
  [EAudioCommand.LIST_DEVICES]: IIpcCommandSignature<typeof audioCommands.listDevices, []>;
  [EAudioCommand.PAUSE_PREVIEW]: IIpcCommandSignature<typeof audioCommands.pausePreview, ["transport"]>;
  [EAudioCommand.PLAY_PREVIEW]: IIpcCommandSignature<typeof audioCommands.playPreview, ["transport"]>;
  [EAudioCommand.READ_PREVIEW_PEAKS]: IIpcCommandSignature<
    typeof audioCommands.readPreviewPeaks,
    ["transport", "count"]
  >;
  [EAudioCommand.SEEK_PREVIEW]: IIpcCommandSignature<typeof audioCommands.seekPreview, ["transport", "position"]>;
  [EAudioCommand.SET_PREVIEW_LOOPED]: IIpcCommandSignature<
    typeof audioCommands.setPreviewLooped,
    ["transport", "isLooped"]
  >;
  [EAudioCommand.SET_SETTINGS]: IIpcCommandSignature<typeof audioCommands.setSettings, ["settings"]>;
  [EAudioCommand.STOP_PREVIEW]: IIpcCommandSignature<typeof audioCommands.stopPreview, ["transport"]>;
  [EAudioCommand.WATCH_PREVIEW]: IIpcCommandSignature<typeof audioCommands.watchPreview, ["transport", "states"]>;
}
