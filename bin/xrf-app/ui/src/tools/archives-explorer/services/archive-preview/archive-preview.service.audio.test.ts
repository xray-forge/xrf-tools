import { beforeEach, describe, expect, it } from "@jest/globals";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { EArchivesBulkRoute } from "@xrf/ipc/commands/archives-bulk";
import { listMockBulkCalls, mockInvoke, mockIpcCommands } from "@xrf/ipc/testing";
import { AudioDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";

import {
  mockArchiveFileDescriptor,
  mockArchiveReadPolicy,
  mockArchivesVolumes,
} from "@/core/archive/fixtures/archive.mocks";
import { mockSessionRead, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockArchivesExplorer } from "@/tools/archives-explorer/fixtures/archives-explorer.mocks";

import { ArchivePreviewService } from "./archive-preview.service";

const SESSION: string = "archives-session";
const TEXTURE: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "textures\\ui.dds" });

const SOUND: ArchiveFileDescriptor = mockArchiveFileDescriptor({
  name: "sounds\\ambient\\wind.ogg",
  sizeCompressed: 4096,
  sizeReal: 8192,
});

const DESCRIPTOR: AudioDescriptor = {
  channels: 2,
  sampleRate: 44100,
  parameters: { minDistance: 1, maxDistance: 50, baseVolume: 0.8, gameType: 3, maxAiDistance: 25 },
};

function mockService(sound: ArchiveFileDescriptor = SOUND): ArchivePreviewService {
  return mockArchivesExplorer(mockSessionSnapshot(mockArchivesVolumes([sound, TEXTURE]), SESSION)).previewService;
}

describe("ArchivePreviewService audio preview", () => {
  beforeEach(() => {
    mockIpcCommands({
      [EArchivesCommand.DESCRIBE_AUDIO]: mockSessionRead(DESCRIPTOR),
    });
  });

  it("does not describe a sound above the audio limit, including on retry", async () => {
    const sound = mockArchiveFileDescriptor({
      ...SOUND,
      sizeReal: mockArchiveReadPolicy().maximumAudioSize + 1,
    });
    const service: ArchivePreviewService = mockService(sound);

    await service.selectFile(sound);
    await service.retry();

    expect(service.selectedEntry).toEqual(sound);
    expect(service.content.value).toBeNull();
    expect(service.content.isLoading).toBe(false);
    expect(service.content.error).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("describes a sound exactly at the audio limit", async () => {
    const sound = mockArchiveFileDescriptor({
      ...SOUND,
      sizeReal: mockArchiveReadPolicy().maximumAudioSize,
    });
    const service: ArchivePreviewService = mockService(sound);

    await service.selectFile(sound);

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_AUDIO, {
      sessionId: SESSION,
      name: sound.name,
    });
    expect(service.content.value?.kind).toBe("audio");
  });

  it("routes a sound to the audio commands rather than reading it as text", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(SOUND);

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_AUDIO, {
      sessionId: SESSION,
      name: SOUND.name,
    });
    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.READ_FILE, expect.anything());
    expect(service.content.value?.kind).toBe("audio");
  });

  it("carries the engine parameters and where the sound is, but none of its bytes", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(SOUND);

    const content = service.content.value?.kind === "audio" ? service.content.value : null;

    // The player opens the sound in the backend by the session and name the description was read by, so the numbers
    // beside it always describe what plays; the page never holds the sound itself.
    expect(content?.descriptor.parameters).toEqual(DESCRIPTOR.parameters);
    expect(content?.sessionId).toBe(SESSION);
    expect(content?.name).toBe(SOUND.name);
    expect(listMockBulkCalls(EArchivesBulkRoute.READ_ENTRY)).toHaveLength(0);
  });

  it("keeps textures on the texture path", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(TEXTURE);

    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_AUDIO, expect.anything());
    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_TEXTURE, expect.anything());
  });

  it("reports a failed read instead of staying loading", async () => {
    const service: ArchivePreviewService = mockService();

    mockIpcCommands({
      [EArchivesCommand.DESCRIBE_AUDIO]: mockSessionRead(() => {
        throw new Error("not a playable sound");
      }),
    });

    await service.selectFile(SOUND);

    expect(service.content.isLoading).toBe(false);
    expect(String(service.content.error)).toContain("not a playable sound");
  });

  it("retries the audio read rather than falling back to text", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(SOUND);
    await service.retry();

    const describeCalls = mockInvoke.mock.calls.filter(([command]) => command === EArchivesCommand.DESCRIBE_AUDIO);

    expect(describeCalls).toHaveLength(2);
  });
});
