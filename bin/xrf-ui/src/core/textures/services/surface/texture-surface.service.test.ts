import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { isObservableProp } from "@wirestate/mobx";
import { mockDdsFile } from "@xrf/renderer/fixtures";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { EMPTY_TEXTURE_SURFACE } from "@/core/textures/lib/texture-surface";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockMaterialDescriptor } from "@/fixtures/mocks/visual.mocks";
import { muteConsole } from "@/fixtures/utils/console";
import { mockInjectedService } from "@/fixtures/utils/container";

/** A four by four DXT1 file, which is the smallest thing `createDdsTexture` will actually upload. */
function mockUploadableTexture(): ArrayBuffer {
  return mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 1, width: 4 });
}

/** A texture whose descriptor declares a pair, so a load reads three files rather than one. */
function mockBumpedDescription(): TextureDescription {
  return mockTextureDescription(MOCK_TEXTURE, { material: mockMaterialDescriptor() });
}

describe("TextureSurfaceService", () => {
  muteConsole("error");

  beforeEach(() => {
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:assets|read_asset"]: mockUploadableTexture() });
  });

  afterEach(() => {
    jest.restoreAllMocks();
  });

  it("applies its mobx annotations", () => {
    // A service whose annotations never got applied still passes every behavioural test here, because nothing in jest
    // reacts to its state - and then does nothing at all in the running app.
    const { service } = mockInjectedService(TextureSurfaceService);

    expect(isObservableProp(service, "files")).toBe(true);
    expect(isObservableProp(service, "reference")).toBe(true);
  });

  it("reads the base and the pair the descriptor declares", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockBumpedDescription());

    const files = service.files.value ?? EMPTY_TEXTURE_SURFACE;

    // Bytes and what the read learned about them; uploading is the drawing side's business.
    expect(files.base?.bytes.byteLength).toBeGreaterThan(0);
    expect(files.base).toMatchObject({ height: 4, isDecoded: false, width: 4 });
    expect(files.bump?.bump.bytes.byteLength).toBeGreaterThan(0);
    expect(files.bump?.companion.bytes.byteLength).toBeGreaterThan(0);
    expect(service.reference).toBe(MOCK_TEXTURE);
  });

  it("reads no pair for a texture declaring none", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockTextureDescription());

    expect((service.files.value ?? EMPTY_TEXTURE_SURFACE).base?.bytes.byteLength).toBeGreaterThan(0);
    expect((service.files.value ?? EMPTY_TEXTURE_SURFACE).bump).toBeNull();
  });

  it("keeps the base and skips the companion when the bump read fails", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);
    const reads: Array<string> = [];

    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: (args?: Record<string, unknown>) => {
        const path: string = String(args?.logicalPath);

        reads.push(path);

        if (path.endsWith("_bump.dds")) {
          throw new Error("Unreadable bump");
        }

        return mockUploadableTexture();
      },
    });

    await service.load(mockBumpedDescription());

    expect(service.files.value?.base?.bytes.byteLength).toBeGreaterThan(0);
    expect(service.files.value?.bump).toBeNull();
    expect(service.bumpTexels).toBeNull();
    expect(service.reference).toBe(MOCK_TEXTURE);
    expect(reads).toEqual([`textures\\${MOCK_TEXTURE}.dds`, "textures\\wpn\\wpn_ak74_bump.dds"]);

    service.clear();
  });

  it("keeps the base and publishes no pair when the companion cannot be read", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    setMockInvokeResponses({
      ["plugin:assets|read_asset"]: (args?: Record<string, unknown>) => {
        if (String(args?.logicalPath).endsWith("_bump#.dds")) {
          throw new Error("Unreadable companion");
        }

        return mockUploadableTexture();
      },
    });

    await service.load(mockBumpedDescription());

    // Both halves or neither: the decode samples the pair every texel, and half of it shades nothing.
    expect(service.files.value?.base?.bytes.byteLength).toBeGreaterThan(0);
    expect(service.files.value?.bump).toBeNull();
    expect(service.bumpTexels).toBeNull();
    expect(service.reference).toBe(MOCK_TEXTURE);
  });

  it("forgets what it read when cleared", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockBumpedDescription());

    expect(service.files.value?.base?.bytes.byteLength).toBeGreaterThan(0);

    service.clear();

    expect(service.files.value ?? EMPTY_TEXTURE_SURFACE).toEqual({ aspect: 1, base: null, bump: null });
    expect(service.reference).toBeNull();
    expect(service.bumpTexels).toBeNull();
  });

  // Nothing here reaches the gpu, so a read the next selection abandoned costs a buffer that is collected and
  // nothing that has to be released. The whole disposal path this service used to carry went with the uploads.
  it("publishes nothing from a run the next selection abandoned", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    // The read is held open, so the run can be cancelled while its file is still being fetched.
    let release!: (bytes: ArrayBuffer) => void;

    const pending: Promise<ArrayBuffer> = new Promise<ArrayBuffer>((resolve) => {
      release = resolve;
    });

    setMockInvokeResponses({ ["plugin:assets|read_asset"]: () => pending });

    const abandoned = service.load(mockTextureDescription());

    service.clear();

    release(mockUploadableTexture());

    await abandoned;

    expect((service.files.value ?? EMPTY_TEXTURE_SURFACE).base).toBeNull();
    expect(service.reference).toBeNull();
  });
});
