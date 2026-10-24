import { beforeEach, describe, expect, it } from "@jest/globals";
import { AsyncState } from "@xrf/async";
import { listMockBulkCalls, mockInvoke, setMockBulkResponses, setMockInvokeResponses } from "@xrf/ipc/testing";
import { AudioDescriptor } from "@xrf/ipc/types/xrf-app-archives";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";

import { mockArchiveFileDescriptor, mockArchiveReadPolicy, mockArchivesVolumes } from "@/fixtures/mocks/archive.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { ArchivesService } from "./archives.service";

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

function mockService(sound: ArchiveFileDescriptor = SOUND): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes([sound, TEXTURE]), SESSION));

  return service;
}

describe("ArchivesService audio preview", () => {
  beforeEach(() => {
    setMockInvokeResponses({
      ["plugin:archives|describe_audio"]: DESCRIPTOR,
    });
    setMockBulkResponses({
      "archives/read_entry": BYTES,
    });
  });

  it("does not describe or read a sound above the audio limit, including on retry", async () => {
    const sound = mockArchiveFileDescriptor({
      ...SOUND,
      sizeReal: mockArchiveReadPolicy().maximumAudioSize + 1,
    });
    const service: ArchivesService = mockService(sound);

    await service.selectArchiveFile(sound);
    await service.retrySelectedFile();

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
    const service: ArchivesService = mockService(sound);

    await service.selectArchiveFile(sound);

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|describe_audio", {
      sessionId: SESSION,
      name: sound.name,
    });
    expect(listMockBulkCalls("archives/read_entry")).toContainEqual({
      sessionId: SESSION,
      name: sound.name,
    });
    expect(service.content.value?.kind).toBe("audio");
  });

  it("routes a sound to the audio commands rather than reading it as text", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(SOUND);

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|describe_audio", {
      sessionId: SESSION,
      name: SOUND.name,
    });
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
    expect(service.content.value?.kind).toBe("audio");
  });

  it("describes and reads one file, of the same subject", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(SOUND);

    // The description and the bytes come from different commands, and nothing but this pairing stops a sound from
    // being played with another file's numbers beside it.
    const [describeArguments] = mockInvoke.mock.calls
      .filter(([command]) => command === "plugin:archives|describe_audio")
      .map(([, args]) => args);
    const [readArguments] = listMockBulkCalls("archives/read_entry");

    expect(describeArguments).toEqual(readArguments);
  });

  it("carries the engine parameters the archive stored, and the bytes beside them", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(SOUND);

    const content = service.content.value?.kind === "audio" ? service.content.value : null;

    // These come from the vorbis comment and are the reason the backend parses at all - the webview
    // could play the bytes without any of it.
    expect(content?.descriptor.parameters).toEqual(DESCRIPTOR.parameters);
    expect(Array.from(content?.bytes ?? [])).toEqual([0x4f, 0x67, 0x67, 0x53]);
  });

  it("keeps textures on the texture path", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(TEXTURE);

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|describe_audio", expect.anything());
    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|describe_texture", expect.anything());
  });

  it("reports a failed read instead of staying loading", async () => {
    const service: ArchivesService = mockService();

    setMockInvokeResponses({
      ["plugin:archives|describe_audio"]: () => {
        throw new Error("not a playable sound");
      },
    });
    setMockBulkResponses({
      "archives/read_entry": BYTES,
    });

    await service.selectArchiveFile(SOUND);

    expect(service.content.isLoading).toBe(false);
    expect(String(service.content.error)).toContain("not a playable sound");
  });

  it("reports a failed byte read even when the description succeeds", async () => {
    const service: ArchivesService = mockService();

    setMockInvokeResponses({
      ["plugin:archives|describe_audio"]: DESCRIPTOR,
    });
    setMockBulkResponses({
      "archives/read_entry": () => {
        throw new Error("resolves to nothing in the open subject");
      },
    });

    await service.selectArchiveFile(SOUND);

    // Half a preview is not a preview: a described sound with no bytes has nothing to play.
    expect(service.content.isLoading).toBe(false);
    expect(service.content.value).toBeNull();
    expect(String(service.content.error)).toContain("resolves to nothing");
  });

  it("retries the audio read rather than falling back to text", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(SOUND);
    await service.retrySelectedFile();

    const describeCalls = mockInvoke.mock.calls.filter(([command]) => command === "plugin:archives|describe_audio");

    expect(describeCalls).toHaveLength(2);
  });
});
