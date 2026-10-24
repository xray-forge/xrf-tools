import { beforeEach, describe, expect, it } from "@jest/globals";
import { AsyncState } from "@xrf/async";
import { listMockBulkCalls, mockInvoke, setMockBulkResponses, setMockInvokeResponses } from "@xrf/ipc/testing";
import { AssetTextureDescriptor } from "@xrf/ipc/types/xrf-app-core";
import { ArchiveFileDescriptor } from "@xrf/ipc/types/xrf-archive";

import { mockArchiveFileDescriptor, mockArchiveReadPolicy, mockArchivesVolumes } from "@/fixtures/mocks/archive.mocks";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";

import { ArchivesService } from "./archives.service";

const SESSION: string = "archives-session";
const BYTES: ArrayBuffer = new Uint8Array([0x89, 0x50, 0x4e, 0x47]).buffer;
const TEXT: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

const TEXTURE: ArchiveFileDescriptor = mockArchiveFileDescriptor({
  name: "textures\\ui\\wall.dds",
  sizeCompressed: 512,
  sizeReal: 2048,
});

const DESCRIPTOR: AssetTextureDescriptor = {
  size: 2048,
  shape: { width: 256, height: 256, mipmapLevels: 9, format: "DXT5" },
};

function mockService(texture: ArchiveFileDescriptor = TEXTURE): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  service["subjectState"] = AsyncState.ready(mockSessionSnapshot(mockArchivesVolumes([texture, TEXT]), SESSION));

  return service;
}

describe("ArchivesService texture preview", () => {
  beforeEach(() => {
    setMockInvokeResponses({
      ["plugin:archives|describe_texture"]: DESCRIPTOR,
    });
    setMockBulkResponses({
      "archives/read_texture": BYTES,
    });
  });

  it("does not describe or decode a texture above the texture limit, including on retry", async () => {
    const texture = mockArchiveFileDescriptor({
      ...TEXTURE,
      sizeReal: mockArchiveReadPolicy().maximumTextureSize + 1,
    });
    const service: ArchivesService = mockService(texture);

    await service.selectArchiveFile(texture);
    await service.retrySelectedFile();

    expect(service.selectedEntry).toEqual(texture);
    expect(service.content.value).toBeNull();
    expect(service.content.isLoading).toBe(false);
    expect(service.content.error).toBeNull();
    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("describes and decodes a texture exactly at the texture limit", async () => {
    const texture = mockArchiveFileDescriptor({
      ...TEXTURE,
      sizeReal: mockArchiveReadPolicy().maximumTextureSize,
    });
    const service: ArchivesService = mockService(texture);

    await service.selectArchiveFile(texture);

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|describe_texture", {
      sessionId: SESSION,
      name: texture.name,
    });
    expect(listMockBulkCalls("archives/read_texture")).toContainEqual({
      sessionId: SESSION,
      name: texture.name,
    });
    expect(service.content.value?.kind).toBe("texture");
  });

  it("decodes a texture instead of reading it as text", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(TEXTURE);

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|describe_texture", {
      sessionId: SESSION,
      name: TEXTURE.name,
    });
    // The text path would have refused it anyway: this entry is compressed and .dds is not readable.
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
    expect(service.content.value?.kind === "texture" ? service.content.value.descriptor.shape?.width : null).toBe(256);
  });

  it("describes and reads one file, of the same subject", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(TEXTURE);

    // The dimensions the viewport lays out against come from the description, the pixels from the read. Addressed
    // apart, they could belong to different subjects.
    const [describeArguments] = mockInvoke.mock.calls
      .filter(([command]) => command === "plugin:archives|describe_texture")
      .map(([, args]) => args);
    const [readArguments] = listMockBulkCalls("archives/read_texture");

    expect(describeArguments).toEqual(readArguments);
  });

  it("carries the decoded png beside the source shape", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(TEXTURE);

    const content = service.content.value?.kind === "texture" ? service.content.value : null;

    expect(content?.descriptor.shape?.format).toBe("DXT5");
    expect(Array.from(content?.bytes ?? [])).toEqual([0x89, 0x50, 0x4e, 0x47]);
  });

  it("leaves text files on the text path", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(TEXT);

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|describe_texture", expect.anything());
    expect(service.content.value?.kind).not.toBe("texture");
  });

  it("reports a failed decode instead of staying loading", async () => {
    const service: ArchivesService = mockService();

    setMockInvokeResponses({
      ["plugin:archives|describe_texture"]: DESCRIPTOR,
    });
    setMockBulkResponses({
      "archives/read_texture": () => {
        throw new Error("unsupported DXT format");
      },
    });

    await service.selectArchiveFile(TEXTURE);

    expect(service.content.isLoading).toBe(false);
    expect(String(service.content.error)).toContain("unsupported DXT format");
  });

  it("retries the decode rather than falling back to a text read", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(TEXTURE);
    await service.retrySelectedFile();

    const imageCalls = listMockBulkCalls("archives/read_texture");

    expect(imageCalls).toHaveLength(2);
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
  });

  it("drops the decoded texture when the selection changes", async () => {
    const service: ArchivesService = mockService();

    await service.selectArchiveFile(TEXTURE);
    expect(service.content.value?.kind).toBe("texture");

    // An image outliving its file would be shown beside the next selection.
    service.selectArchiveDirectory("textures");
    expect(service.content.value).toBeNull();
  });
});
