import { describe, expect, it } from "@jest/globals";
import { Texture } from "three/webgpu";

import { mockDdsFile, mockUncompressedDdsFile } from "#/dds/dds-fixtures";
import { ITextureDeviceFixture, mockTextureDevice } from "#/internals/device-fixtures";
import { createRendererRawTexture, createRendererTexture } from "#/texture/renderer-texture";
import { hasTextureData, listTextureData, releaseTextureData } from "#/texture/texture-data";

function createBlockTexture(): Texture {
  return createRendererTexture(mockDdsFile({ fourCC: "DXT5", height: 8, mipmapCount: 2, width: 8 })).texture as Texture;
}

describe("releaseTextureData", () => {
  it("lets go of every level's bytes, keeping each level's size and how many there are", () => {
    const texture: Texture = createBlockTexture();

    expect(listTextureData(texture).map((data: ArrayBufferView) => data.byteLength)).toEqual([64, 16]);

    releaseTextureData(texture);

    expect(listTextureData(texture)).toEqual([]);
    expect(texture.mipmaps).toEqual([
      { data: null, height: 8, width: 8 },
      { data: null, height: 4, width: 4 },
    ]);
    expect(texture.image).toEqual({ data: null, height: 8, width: 8 });
    expect(hasTextureData(texture)).toBe(false);
  });

  it("lets go of the image of a texture of texels, read or made", () => {
    const texture: Texture = createRendererTexture(mockUncompressedDdsFile({ height: 4, width: 4 })).texture as Texture;
    const raw: Texture = createRendererRawTexture(new ArrayBuffer(16), 2, 2);

    expect(listTextureData(texture).length).toBeGreaterThan(0);

    releaseTextureData(texture);
    releaseTextureData(raw);

    expect(listTextureData(texture)).toEqual([]);
    expect(listTextureData(raw)).toEqual([]);
    expect(raw.image).toEqual({ data: null, height: 2, width: 2 });
  });

  // A binding built over it before it was disposed has three make it again, which would throw on bytes that are gone.
  it("leaves a texture three makes again after it was disposed allocating alone, reading nothing", () => {
    const device: ITextureDeviceFixture = mockTextureDevice();
    const texture: Texture = createBlockTexture();

    device.renderer.initTexture(texture);
    releaseTextureData(texture);
    texture.dispose();

    expect(() => device.renderer.initTexture(texture)).not.toThrow();
    expect(device.uploads.map(([, bytes]: [Texture, number]) => bytes)).toEqual([80, 0]);
  });
});
