import { beforeEach, describe, expect, it } from "@jest/globals";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { EArchivesBulkRoute } from "@xrf/ipc/commands/archives-bulk";
import { listMockBulkCalls, mockInvoke, mockIpcBulkRoutes, mockIpcCommands } from "@xrf/ipc/testing";
import { AudioDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";

import { mockArchivesExplorer } from "@/applications/archives-explorer/fixtures/archives-explorer.mocks";
import {
  mockArchiveFileDescriptor,
  mockArchiveReadPolicy,
  mockArchivesVolumes,
} from "@/core/archive/fixtures/archive.mocks";
import { mockSessionRead, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";

import { ArchivePreviewService } from "./archive-preview.service";

const SESSION: string = "archives-session";
const BYTES: ArrayBuffer = new Uint8Array([0x4f, 0x67, 0x67, 0x53]).buffer;
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
  mediaType: "audio/ogg",
};

function mockService(sound: ArchiveFileDescriptor = SOUND): ArchivePreviewService {
  return mockArchivesExplorer(mockSessionSnapshot(mockArchivesVolumes([sound, TEXTURE]), SESSION)).previewService;
}

describe("ArchivePreviewService audio preview", () => {
  beforeEach(() => {
    mockIpcCommands({
      [EArchivesCommand.DESCRIBE_AUDIO]: mockSessionRead(DESCRIPTOR),
    });
    mockIpcBulkRoutes({
      [EArchivesBulkRoute.READ_ENTRY]: BYTES,
    });
  });

  it("does not describe or read a sound above the audio limit, including on retry", async () => {
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

  it("describes and reads a sound exactly at the audio limit", async () => {
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
    expect(listMockBulkCalls(EArchivesBulkRoute.READ_ENTRY)).toContainEqual({
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

  it("describes and reads one file, of the same subject", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(SOUND);

    // The description and the bytes come from different commands, and nothing but this pairing stops a sound from
    // being played with another file's numbers beside it.
    const [describeArguments] = mockInvoke.mock.calls
      .filter(([command]) => command === EArchivesCommand.DESCRIBE_AUDIO)
      .map(([, args]) => args);
    const [readArguments] = listMockBulkCalls(EArchivesBulkRoute.READ_ENTRY);

    expect(describeArguments).toEqual(readArguments);
  });

  it("carries the engine parameters the archive stored, and the bytes beside them", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(SOUND);

    const content = service.content.value?.kind === "audio" ? service.content.value : null;

    // These come from the vorbis comment and are the reason the backend parses at all - the webview
    // could play the bytes without any of it.
    expect(content?.descriptor.parameters).toEqual(DESCRIPTOR.parameters);
    expect(Array.from(content?.bytes ?? [])).toEqual([0x4f, 0x67, 0x67, 0x53]);
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
    mockIpcBulkRoutes({
      [EArchivesBulkRoute.READ_ENTRY]: BYTES,
    });

    await service.selectFile(SOUND);

    expect(service.content.isLoading).toBe(false);
    expect(String(service.content.error)).toContain("not a playable sound");
  });

  it("reports a failed byte read even when the description succeeds", async () => {
    const service: ArchivePreviewService = mockService();

    mockIpcCommands({
      [EArchivesCommand.DESCRIBE_AUDIO]: mockSessionRead(DESCRIPTOR),
    });
    mockIpcBulkRoutes({
      [EArchivesBulkRoute.READ_ENTRY]: () => {
        throw new Error("resolves to nothing in the open subject");
      },
    });

    await service.selectFile(SOUND);

    // Half a preview is not a preview: a described sound with no bytes has nothing to play.
    expect(service.content.isLoading).toBe(false);
    expect(service.content.value).toBeNull();
    expect(String(service.content.error)).toContain("resolves to nothing");
  });

  it("retries the audio read rather than falling back to text", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(SOUND);
    await service.retry();

    const describeCalls = mockInvoke.mock.calls.filter(([command]) => command === EArchivesCommand.DESCRIBE_AUDIO);

    expect(describeCalls).toHaveLength(2);
  });
});
