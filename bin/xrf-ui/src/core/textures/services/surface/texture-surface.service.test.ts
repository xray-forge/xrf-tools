import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { isObservableProp } from "@wirestate/mobx";
import { Texture } from "three";

import { TextureDescription } from "@/core/bindings/types/xrf-app";
import { EMPTY_TEXTURE_SURFACE } from "@/core/textures/lib/texture-surface";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { mockDdsFile } from "@/fixtures/mocks/dds.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";
import { mockMaterialDescriptor } from "@/fixtures/mocks/visual.mocks";
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
  beforeEach(() => {
    resetMockInvoke();
    setMockInvokeResponses({ ["plugin:assets|read_asset"]: mockUploadableTexture() });
  });

  it("applies its mobx annotations", () => {
    // A service whose annotations never got applied still passes every behavioural test here, because nothing in jest
    // reacts to its state - and then does nothing at all in the running app.
    const { service } = mockInjectedService(TextureSurfaceService);

    expect(isObservableProp(service, "textures")).toBe(true);
    expect(isObservableProp(service, "uploaded")).toBe(true);
  });

  it("uploads the base and the pair the descriptor declares", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockBumpedDescription());

    expect((service.textures.value ?? EMPTY_TEXTURE_SURFACE).base).toBeInstanceOf(Texture);
    expect((service.textures.value ?? EMPTY_TEXTURE_SURFACE).bump?.bump).toBeInstanceOf(Texture);
    expect((service.textures.value ?? EMPTY_TEXTURE_SURFACE).bump?.companion).toBeInstanceOf(Texture);
    expect(service.uploaded).toBe(MOCK_TEXTURE);
  });

  it("uploads no pair for a texture declaring none", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockTextureDescription());

    expect((service.textures.value ?? EMPTY_TEXTURE_SURFACE).base).toBeInstanceOf(Texture);
    expect((service.textures.value ?? EMPTY_TEXTURE_SURFACE).bump).toBeNull();
  });

  it("releases what it uploaded when cleared", async () => {
    const { service } = mockInjectedService(TextureSurfaceService);

    await service.load(mockBumpedDescription());

    const released: Array<Texture> = [];

    for (const texture of [
      (service.textures.value ?? EMPTY_TEXTURE_SURFACE).base,
      (service.textures.value ?? EMPTY_TEXTURE_SURFACE).bump?.bump,
      (service.textures.value ?? EMPTY_TEXTURE_SURFACE).bump?.companion,
    ]) {
      texture?.addEventListener("dispose", () => released.push(texture));
    }

    service.clear();

    expect(released).toHaveLength(3);
    expect(service.textures.value ?? EMPTY_TEXTURE_SURFACE).toEqual({ aspect: 1, base: null, bump: null });
    expect(service.uploaded).toBeNull();
  });

  it("releases the reads of a run the next selection abandoned", async () => {
    // The read is still in flight when the run is cancelled, so its texture is uploaded after the run has left. Left
    // alone it is gpu memory nothing holds a reference to and no surface will ever draw.
    const dispose = jest.spyOn(Texture.prototype, "dispose");
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
    // The read resolves, decodes and is released over several turns, none of which the cancelled run is waiting on.
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(dispose).toHaveBeenCalled();
    expect((service.textures.value ?? EMPTY_TEXTURE_SURFACE).base).toBeNull();

    dispose.mockRestore();
  });
});
