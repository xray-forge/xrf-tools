import { beforeEach, describe, expect, it } from "@jest/globals";
import { EAudioCommand } from "@xrf/ipc/commands/audio";
import { mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { AudioSettings } from "@xrf/ipc/types/xrf-app-audio";
import { Nullable } from "@xrf/types";

import { AudioSettingsService } from "@/core/audio/services/audio-settings.service";
import { mockInjectedService } from "@/fixtures/utils/container";

const SETTINGS: AudioSettings = {
  output: { kind: "systemDefault" },
  volumes: { master: 1, music: 1, ambience: 1, world: 1, previews: 1 },
  isMutedInBackground: true,
};

async function mockReadService(): Promise<AudioSettingsService> {
  const service: AudioSettingsService = mockInjectedService(AudioSettingsService).service;

  await service.read();

  return service;
}

describe("AudioSettingsService", () => {
  beforeEach(() => {
    mockIpcCommands({
      [EAudioCommand.DESCRIBE_SETTINGS]: SETTINGS,
      [EAudioCommand.SET_SETTINGS]: ({ settings }: { settings: AudioSettings }) => settings,
    });
  });

  it("reads the settings the backend keeps, at full volume before it answers", async () => {
    const service: AudioSettingsService = mockInjectedService(AudioSettingsService).service;

    expect(service.previewsVolume).toBe(1);

    await service.read();

    expect(service.settings).toEqual(SETTINGS);
  });

  it("sends a volume with the rest of the settings, held in range", async () => {
    const service: AudioSettingsService = await mockReadService();

    service.setVolume("music", 1.5);
    service.setPreviewsVolume(0.25);

    expect(mockInvoke).toHaveBeenCalledWith(EAudioCommand.SET_SETTINGS, {
      settings: { ...SETTINGS, volumes: { ...SETTINGS.volumes, music: 1, previews: 0.25 } },
    });
    expect(service.getVolume("previews")).toBe(0.25);
  });

  it("keeps the latest write's value when an earlier one answers last", async () => {
    const first: { answer: Nullable<(settings: AudioSettings) => void> } = { answer: null };
    const service: AudioSettingsService = await mockReadService();

    mockIpcCommands({
      [EAudioCommand.SET_SETTINGS]: ({ settings }: { settings: AudioSettings }) =>
        settings.volumes.master === 0.2 ? new Promise<AudioSettings>((resolve) => (first.answer = resolve)) : settings,
    });

    service.setVolume("master", 0.2);
    service.setVolume("master", 0.8);

    await Promise.resolve();
    first.answer?.({ ...SETTINGS, volumes: { ...SETTINGS.volumes, master: 0.2 } });
    await new Promise((resolve) => setTimeout(resolve));

    expect(service.getVolume("master")).toBe(0.8);
  });

  it("sends the chosen device and the background mute", async () => {
    const service: AudioSettingsService = await mockReadService();

    service.setOutput({ kind: "device", id: "speakers" });
    service.setMutedInBackground(false);

    expect(mockInvoke).toHaveBeenCalledWith(EAudioCommand.SET_SETTINGS, {
      settings: { ...SETTINGS, output: { kind: "device", id: "speakers" } },
    });
    expect(service.settings?.isMutedInBackground).toBe(false);
  });
});
