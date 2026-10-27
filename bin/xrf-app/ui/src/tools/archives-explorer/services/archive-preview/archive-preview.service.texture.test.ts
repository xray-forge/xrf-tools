import { beforeEach, describe, expect, it } from "@jest/globals";
import { EArchivesCommand } from "@xrf/ipc/commands/archives";
import { EArchivesBulkRoute } from "@xrf/ipc/commands/archives-bulk";
import { listMockBulkCalls, mockInvoke, mockIpcBulkRoutes, mockIpcCommands } from "@xrf/ipc/testing";
import { AssetTextureDescriptor } from "@xrf/ipc/types/xrf-app-core";
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

function mockService(texture: ArchiveFileDescriptor = TEXTURE): ArchivePreviewService {
  return mockArchivesExplorer(mockSessionSnapshot(mockArchivesVolumes([texture, TEXT]), SESSION)).previewService;
}

describe("ArchivePreviewService texture preview", () => {
  beforeEach(() => {
    mockIpcCommands({
      [EArchivesCommand.DESCRIBE_TEXTURE]: mockSessionRead(DESCRIPTOR),
    });
    mockIpcBulkRoutes({
      [EArchivesBulkRoute.READ_TEXTURE]: BYTES,
    });
  });

  it("does not describe or decode a texture above the texture limit, including on retry", async () => {
    const texture = mockArchiveFileDescriptor({
      ...TEXTURE,
      sizeReal: mockArchiveReadPolicy().maximumTextureSize + 1,
    });
    const service: ArchivePreviewService = mockService(texture);

    await service.selectFile(texture);
    await service.retry();

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
    const service: ArchivePreviewService = mockService(texture);

    await service.selectFile(texture);

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_TEXTURE, {
      sessionId: SESSION,
      name: texture.name,
    });
    expect(listMockBulkCalls(EArchivesBulkRoute.READ_TEXTURE)).toContainEqual({
      sessionId: SESSION,
      name: texture.name,
    });
    expect(service.content.value?.kind).toBe("texture");
  });

  it("decodes a texture instead of reading it as text", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(TEXTURE);

    expect(mockInvoke).toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_TEXTURE, {
      sessionId: SESSION,
      name: TEXTURE.name,
    });
    // The text path would have refused it anyway: this entry is compressed and .dds is not readable.
    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.READ_FILE, expect.anything());
    expect(service.content.value?.kind === "texture" ? service.content.value.descriptor.shape?.width : null).toBe(256);
  });

  it("describes and reads one file, of the same subject", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(TEXTURE);

    // The dimensions the viewport lays out against come from the description, the pixels from the read. Addressed
    // apart, they could belong to different subjects.
    const [describeArguments] = mockInvoke.mock.calls
      .filter(([command]) => command === EArchivesCommand.DESCRIBE_TEXTURE)
      .map(([, args]) => args);
    const [readArguments] = listMockBulkCalls(EArchivesBulkRoute.READ_TEXTURE);

    expect(describeArguments).toEqual(readArguments);
  });

  it("carries the decoded png beside the source shape", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(TEXTURE);

    const content = service.content.value?.kind === "texture" ? service.content.value : null;

    expect(content?.descriptor.shape?.format).toBe("DXT5");
    expect(Array.from(content?.bytes ?? [])).toEqual([0x89, 0x50, 0x4e, 0x47]);
  });

  it("leaves text files on the text path", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(TEXT);

    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.DESCRIBE_TEXTURE, expect.anything());
    expect(service.content.value?.kind).not.toBe("texture");
  });

  it("reports a failed decode instead of staying loading", async () => {
    const service: ArchivePreviewService = mockService();

    mockIpcCommands({
      [EArchivesCommand.DESCRIBE_TEXTURE]: mockSessionRead(DESCRIPTOR),
    });
    mockIpcBulkRoutes({
      [EArchivesBulkRoute.READ_TEXTURE]: () => {
        throw new Error("unsupported DXT format");
      },
    });

    await service.selectFile(TEXTURE);

    expect(service.content.isLoading).toBe(false);
    expect(String(service.content.error)).toContain("unsupported DXT format");
  });

  it("retries the decode rather than falling back to a text read", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(TEXTURE);
    await service.retry();

    const imageCalls = listMockBulkCalls(EArchivesBulkRoute.READ_TEXTURE);

    expect(imageCalls).toHaveLength(2);
    expect(mockInvoke).not.toHaveBeenCalledWith(EArchivesCommand.READ_FILE, expect.anything());
  });

  it("drops the decoded texture when the selection changes", async () => {
    const service: ArchivePreviewService = mockService();

    await service.selectFile(TEXTURE);
    expect(service.content.value?.kind).toBe("texture");

    // An image outliving its file would be shown beside the next selection.
    service.selectDirectory("textures");
    expect(service.content.value).toBeNull();
  });
});
