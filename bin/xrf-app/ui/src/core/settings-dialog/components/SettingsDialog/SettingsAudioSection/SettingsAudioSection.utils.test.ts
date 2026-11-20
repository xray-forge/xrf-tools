import { describe, expect, it } from "@jest/globals";
import { AudioDeviceDescription, AudioOutputStatus } from "@xrf/ipc/types/xrf-audio";

import {
  describeAudioOutput,
  fromAudioOutputKey,
  listAudioOutputOptions,
  toAudioOutputKey,
} from "./SettingsAudioSection.utils";

const SPEAKERS: AudioDeviceDescription = { id: "speakers", name: "Speakers", isDefault: true };

const PLAYING: AudioOutputStatus = {
  deviceName: "Speakers",
  sampleRate: 48000,
  channels: 2,
  isFallback: false,
  failure: null,
};

describe("SettingsAudioSection utils", () => {
  it("round-trips every output choice through the row's key", () => {
    expect(fromAudioOutputKey(toAudioOutputKey({ kind: "systemDefault" }))).toEqual({ kind: "systemDefault" });
    expect(fromAudioOutputKey(toAudioOutputKey({ kind: "device", id: "a:b" }))).toEqual({ kind: "device", id: "a:b" });
  });

  it("offers the system default and every device, and keeps a missing chosen device on the row", () => {
    expect(listAudioOutputOptions([SPEAKERS], { kind: "systemDefault" }).map((it) => it.label)).toEqual([
      "System default",
      "Speakers",
    ]);
    expect(listAudioOutputOptions(null, { kind: "device", id: "headphones" }).map((it) => it.label)).toEqual([
      "System default",
      "Missing device",
    ]);
  });

  it("says which device is the system's default now, and what a missing one plays", () => {
    const headphones: AudioDeviceDescription = { id: "headphones", name: "Headphones", isDefault: false };

    expect(
      listAudioOutputOptions([SPEAKERS, headphones], { kind: "systemDefault" }).map((it) => it.description)
    ).toEqual(["Follows the device the system plays on, as it changes", "The system's default device now", undefined]);
    expect(listAudioOutputOptions(null, { kind: "device", id: "gone" })[1].description).toContain(
      "the system default plays"
    );
  });

  it("says what plays, what fell back, and why nothing plays", () => {
    expect(describeAudioOutput(PLAYING)).toContain("Playing on Speakers at 48000 Hz.");
    expect(describeAudioOutput({ ...PLAYING, isFallback: true })).toContain("chosen device is missing");
    expect(describeAudioOutput({ ...PLAYING, deviceName: null, failure: "No output device" })).toContain(
      "No device plays: No output device"
    );
  });
});
