import { afterEach, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { isObservableProp } from "@wirestate/mobx";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { EMPTY_TEXTURE_SURFACE } from "@/core/textures/lib/texture-surface";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { listMockBulkCalls, setMockBulkResponses } from "@/fixtures/mocks/bulk.mocks";
import { resetMockInvoke } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockMaterialDescriptor } from "@/fixtures/mocks/visual.mocks";
import { muteConsole } from "@/fixtures/utils/console";
import { mockInjectedService } from "@/fixtures/utils/container";

/** Texels as `textures/read_texels` answers them: the size, then four bytes a texel. */
function mockTexels(width: number = 2, height: number = 2): ArrayBuffer {
  const buffer: ArrayBuffer = new ArrayBuffer(8 + width * height * 4);
  const bytes: Uint8Array = new Uint8Array(buffer);
  const header: DataView = new DataView(buffer);

  header.setUint32(0, width, true);
  header.setUint32(4, height, true);
  bytes.fill(7, 8);

  return buffer;
}

/** A texture whose descriptor declares a pair, so a load reads both halves. */
function mockBumpedDescription(): TextureDescription {
  return mockTextureDescription(MOCK_TEXTURE, { material: mockMaterialDescriptor() });
}

describe("TextureSurfaceService", () => {
  muteConsole("error");

  beforeEach(() => {
    resetMockInvoke();
    setMockBulkResponses({ "textures/read_texels": mockTexels() });
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

  it("reads the pair the descriptor declares as texels, and never the base, which the renderer reads", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockBumpedDescription());

    const files = service.files.value ?? EMPTY_TEXTURE_SURFACE;

    expect(files.bump?.bump).toMatchObject({ height: 2, width: 2 });
    expect(files.bump?.companion.data).toHaveLength(16);
    expect(listMockBulkCalls("textures/read_texels")).toHaveLength(2);
    expect(listMockBulkCalls("assets/read_asset")).toHaveLength(0);
    expect(service.reference).toBe(MOCK_TEXTURE);
  });

  it("reads nothing for a texture declaring no pair", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockTextureDescription());

    expect(service.files.value?.bump).toBeNull();
    expect(listMockBulkCalls("textures/read_texels")).toHaveLength(0);
  });

  it("publishes no pair when a half cannot be read", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);
    let calls: number = 0;

    setMockBulkResponses({
      "textures/read_texels": () => {
        calls += 1;

        if (calls === 2) {
          throw new Error("companion is truncated");
        }

        return mockTexels();
      },
    });

    await service.load(mockBumpedDescription());

    expect(service.files.value?.bump).toBeNull();
    expect(service.files.error).toBeNull();
  });

  it("forgets what it read when cleared", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockBumpedDescription());

    expect(service.files.value?.bump).not.toBeNull();

    service.clear();

    expect(service.files.value ?? EMPTY_TEXTURE_SURFACE).toEqual({ aspect: 1, bump: null });
    expect(service.reference).toBeNull();
  });

  it("publishes nothing from a run the next selection abandoned", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    // The read is held open, so the run can be cancelled while its file is still being fetched.
    let release!: (bytes: ArrayBuffer) => void;

    const pending: Promise<ArrayBuffer> = new Promise<ArrayBuffer>((resolve) => {
      release = resolve;
    });

    setMockBulkResponses({ "textures/read_texels": () => pending });

    const abandoned = service.load(mockBumpedDescription());

    service.clear();

    release(mockTexels());

    await abandoned;

    expect((service.files.value ?? EMPTY_TEXTURE_SURFACE).bump).toBeNull();
    expect(service.reference).toBeNull();
  });
});
