import { beforeEach, describe, expect, it } from "@jest/globals";

import { ArchivesService } from "@/applications/archives-explorer/services/archives/archives.service";
import { createRoots } from "@/core/assets/lib";
import { AssetTextureDescriptor } from "@/core/bindings/types/xrf-app";
import { ArchiveFileDescriptor } from "@/core/bindings/types/xrf-archive";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { mockArchiveFileDescriptor, mockArchivesProject } from "@/fixtures/mocks/archive.mocks";
import { mockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { createLoadable } from "@/lib/loadable";

const TEXTURE: ArchiveFileDescriptor = mockArchiveFileDescriptor({
  extension: "dds",
  name: "textures\\ui\\wall.dds",
  sizeCompressed: 512,
  sizeReal: 2048,
});

const TEXT: ArchiveFileDescriptor = mockArchiveFileDescriptor({ name: "configs\\system.ltx" });

const DESCRIPTOR: AssetTextureDescriptor = {
  size: 2048,
  shape: { width: 256, height: 256, mipmapLevels: 9, format: "DXT5" },
};

/** The roots an archive project mounts, which both media calls have to name identically. */
const ROOTS: XrayRoots = createRoots(["C:\\game\\database"]);

const BYTES: ArrayBuffer = new Uint8Array([0x89, 0x50, 0x4e, 0x47]).buffer;

/**
 * Creates an archive service with fixture files classified by its open project.
 *
 * @returns Service ready to preview the fixture files.
 */
function createService(): ArchivesService {
  const { service } = mockInjectedService(ArchivesService);

  service.project = createLoadable(mockArchivesProject([TEXTURE, TEXT]));

  return service;
}

describe("ArchivesService image preview", () => {
  beforeEach(() => {
    setMockInvokeResponses({
      ["plugin:archives|describe_image"]: DESCRIPTOR,
      ["plugin:archives|read_image"]: BYTES,
    });
  });

  it("decodes a texture instead of reading it as text", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(TEXTURE);

    expect(mockInvoke).toHaveBeenCalledWith("plugin:archives|describe_image", {
      roots: ROOTS,
      logicalPath: TEXTURE.name,
    });
    // The text path would have refused it anyway: this entry is compressed and .dds is not readable.
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
    expect(service.content.value?.kind === "image" ? service.content.value.descriptor.shape?.width : null).toBe(256);
  });

  it("describes and reads one file, from the same roots", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(TEXTURE);

    // The dimensions the viewport lays out against come from the description, the pixels from the read. Addressed
    // apart, they could belong to different volumes of the same tree.
    const [describeArguments] = mockInvoke.mock.calls
      .filter(([command]) => command === "plugin:archives|describe_image")
      .map(([, args]) => args);
    const [readArguments] = mockInvoke.mock.calls
      .filter(([command]) => command === "plugin:archives|read_image")
      .map(([, args]) => args);

    expect(describeArguments).toEqual(readArguments);
  });

  it("carries the decoded png beside the source shape", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(TEXTURE);

    const content = service.content.value?.kind === "image" ? service.content.value : null;

    expect(content?.descriptor.shape?.format).toBe("DXT5");
    expect(Array.from(content?.bytes ?? [])).toEqual([0x89, 0x50, 0x4e, 0x47]);
  });

  it("leaves text files on the text path", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(TEXT);

    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|describe_image", expect.anything());
    expect(service.content.value?.kind).not.toBe("image");
  });

  it("reports a failed decode instead of staying loading", async () => {
    const service: ArchivesService = createService();

    setMockInvokeResponses({
      ["plugin:archives|describe_image"]: DESCRIPTOR,
      ["plugin:archives|read_image"]: () => {
        throw new Error("unsupported DXT format");
      },
    });

    await service.selectArchiveFile(TEXTURE);

    expect(service.content.isLoading).toBe(false);
    expect(String(service.content.error)).toContain("unsupported DXT format");
  });

  it("retries the decode rather than falling back to a text read", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(TEXTURE);
    await service.retrySelectedFile();

    const imageCalls = mockInvoke.mock.calls.filter(([command]) => command === "plugin:archives|read_image");

    expect(imageCalls).toHaveLength(2);
    expect(mockInvoke).not.toHaveBeenCalledWith("plugin:archives|read_file", expect.anything());
  });

  it("drops the decoded image when the selection changes", async () => {
    const service: ArchivesService = createService();

    await service.selectArchiveFile(TEXTURE);
    expect(service.content.value?.kind).toBe("image");

    // An image outliving its file would be shown beside the next selection.
    service.selectArchiveDirectory("textures");
    expect(service.content.value).toBeNull();
  });
});
