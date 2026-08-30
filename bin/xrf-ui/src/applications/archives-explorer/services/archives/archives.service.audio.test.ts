import { beforeEach, describe, expect, it } from "@jest/globals";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/archives.service";
import { createRoot } from "@/core/assets/lib";
import { AudioDescriptor } from "@/core/bindings/types/xrf-app";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { mockArchiveFileDescriptor, mockArchivesProject } from "@/fixtures/mocks/archive.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { createLoadable } from "@/lib/loadable";

const SOUND: ArchiveFileDescriptor = mockArchiveFileDescriptor({
  extension: "ogg",
  name: "sounds\\ambient\\wind.ogg",
  sizeCompressed: 4096,
  sizeReal: 8192,
});

const TEXTURE: ArchiveFileDescriptor = mockArchiveFileDescriptor({ extension: "dds", name: "textures\\ui.dds" });

const DESCRIPTOR: AudioDescriptor = {
  channels: 2,
  sampleRate: 44100,
  parameters: { minDistance: 1, maxDistance: 50, baseVolume: 0.8, gameType: 3, maxAiDistance: 25 },
};

/**
 * The roots an archive project mounts, which both media calls have to name identically.
 *
 * Read as `volumes`, so a volume in a subdirectory of the project root is searched too.
 */
const ROOTS: XrayRoots = { asset: null, roots: [createRoot("C:\\game\\database", "volumes")] };

const BYTES: ArrayBuffer = new Uint8Array([0x4f, 0x67, 0x67, 0x53]).buffer;

function createService(): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  service.project = createLoadable(mockArchivesProject([SOUND, TEXTURE]));

  return service;
}

describe("ArchivesService audio preview", () => {
  beforeEach(() => {
    setMockInvokeResponses({
      ["plugin:archives|describe_audio"]: DESCRIPTOR,
      ["plugin:assets|read_asset"]: BYTES,
    });
  });

  it("routes a sound to the audio commands rather than reading it as text", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(SOUND);

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|describe_audio", {
      roots: ROOTS,
      logicalPath: SOUND.name,
    });
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
    expect(service.content.value?.kind).toBe("audio");
  });

  it("describes and reads one file, from the same roots", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(SOUND);

    // The description and the bytes come from different commands, and nothing but this pairing stops a sound from
    // being played with another file's numbers beside it.
    const [describeArguments] = mockInvoke.mock.calls
      .filter(([command]) => command === "plugin:archives|describe_audio")
      .map(([, args]) => args);
    const [readArguments] = mockInvoke.mock.calls
      .filter(([command]) => command === "plugin:assets|read_asset")
      .map(([, args]) => args);

    expect(describeArguments).toEqual(readArguments);
  });

  it("carries the engine parameters the archive stored, and the bytes beside them", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(SOUND);

    const content = service.content.value?.kind === "audio" ? service.content.value : null;

    // These come from the vorbis comment and are the reason the backend parses at all - the webview
    // could play the bytes without any of it.
    expect(content?.descriptor.parameters).toEqual(DESCRIPTOR.parameters);
    expect(Array.from(content?.bytes ?? [])).toEqual([0x4f, 0x67, 0x67, 0x53]);
  });

  it("keeps textures on the image path", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(TEXTURE);

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|describe_audio", expect.anything());
    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|describe_image", expect.anything());
  });

  it("reports a failed read instead of staying loading", async () => {
    const service: ArchivesService = createService();

    setMockInvokeResponses({
      ["plugin:archives|describe_audio"]: () => {
        throw new Error("not a playable sound");
      },
      ["plugin:assets|read_asset"]: BYTES,
    });

    await service.selectArchiveFile(SOUND);

    expect(service.content.isLoading).toBe(false);
    expect(String(service.content.error)).toContain("not a playable sound");
  });

  it("reports a failed byte read even when the description succeeds", async () => {
    const service: ArchivesService = createService();

    setMockInvokeResponses({
      ["plugin:archives|describe_audio"]: DESCRIPTOR,
      ["plugin:assets|read_asset"]: () => {
        throw new Error("resolves to nothing in the mounted roots");
      },
    });

    await service.selectArchiveFile(SOUND);

    // Half a preview is not a preview: a described sound with no bytes has nothing to play.
    expect(service.content.isLoading).toBe(false);
    expect(service.content.value).toBeNull();
    expect(String(service.content.error)).toContain("resolves to nothing");
  });

  it("retries the audio read rather than falling back to text", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(SOUND);
    await service.retrySelectedFile();

    const describeCalls = mockInvoke.mock.calls.filter(([command]) => command === "plugin:archives|describe_audio");

    expect(describeCalls).toHaveLength(2);
  });
});
