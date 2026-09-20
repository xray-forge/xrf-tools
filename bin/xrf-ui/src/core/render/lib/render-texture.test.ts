import { afterEach, describe, expect, it, jest } from "@jest/globals";
import {
  CompressedTexture,
  LinearFilter,
  NoColorSpace,
  RepeatWrapping,
  RGB_S3TC_DXT1_Format,
  RGBA_S3TC_DXT1_Format,
  RGBA_S3TC_DXT5_Format,
  SRGBColorSpace,
  Texture,
} from "three";

import { createDdsTexture, createDecodedTexture } from "@/core/render/lib/render-texture";
import { mockDdsFile, mockDx10DdsFile, mockUncompressedDdsFile } from "@/fixtures/mocks/dds.mocks";
import { muteConsole } from "@/fixtures/utils/console";
import { Nullable } from "@/lib/types/general";

describe("createDecodedTexture", () => {
  const originalDecoder = Object.getOwnPropertyDescriptor(globalThis, "createImageBitmap");

  afterEach(() => {
    if (originalDecoder) {
      Object.defineProperty(globalThis, "createImageBitmap", originalDecoder);
    } else {
      Reflect.deleteProperty(globalThis, "createImageBitmap");
    }
  });

  it("keeps its bitmap available until the texture is disposed", async () => {
    const close = jest.fn();
    const bitmap = { close, height: 4, width: 4 };

    Object.defineProperty(globalThis, "createImageBitmap", {
      configurable: true,
      value: async () => bitmap,
    });

    const texture: Texture = await createDecodedTexture(new ArrayBuffer(0));

    expect(texture.image).toBe(bitmap);
    expect(close).not.toHaveBeenCalled();

    texture.dispose();

    expect(close).toHaveBeenCalledTimes(1);
  });
});

describe("createDdsTexture", () => {
  // Two of these hand the loader a format it has no branch for, and it reports each refusal itself.
  muteConsole("error");

  it("uploads a dxt1 file with its mip chain", () => {
    const texture: Nullable<CompressedTexture> = createDdsTexture(
      mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 3, width: 4 })
    );

    expect(texture).not.toBeNull();
    expect(texture!.format).toBe(RGB_S3TC_DXT1_Format);
    expect(texture!.image.width).toBe(4);
    expect(texture!.mipmaps).toHaveLength(3);
  });

  it("keeps dxt1's alpha bit for a surface that reads alpha, and drops it otherwise", () => {
    const read: Nullable<CompressedTexture> = createDdsTexture(mockDdsFile({ fourCC: "DXT1" }), { isAlphaRead: true });
    const ignored: Nullable<CompressedTexture> = createDdsTexture(mockDdsFile({ fourCC: "DXT1" }), {
      isAlphaRead: false,
    });

    expect(read!.format).toBe(RGBA_S3TC_DXT1_Format);
    // Not applied always: the transparent-black block mode occurs in files authored opaque, and reading those as rgba
    // would punch holes in surfaces the engine draws solid.
    expect(ignored!.format).toBe(RGB_S3TC_DXT1_Format);
  });

  it("leaves a format that already carries alpha alone", () => {
    expect(createDdsTexture(mockDdsFile({ fourCC: "DXT5" }), { isAlphaRead: true })!.format).toBe(
      RGBA_S3TC_DXT5_Format
    );
  });

  it("drops to a non mipmap filter when the file carries no mip chain", () => {
    // Load bearing rather than cosmetic: webgl samples an incomplete texture as black, and most modded textures ship
    // without mips - 1,805 of Anomaly's 2,197.
    const withMips: Nullable<CompressedTexture> = createDdsTexture(mockDdsFile({ mipmapCount: 4 }));
    const withoutMips: Nullable<CompressedTexture> = createDdsTexture(mockDdsFile({ mipmapCount: 1 }));

    expect(withoutMips!.minFilter).toBe(LinearFilter);
    expect(withMips!.minFilter).not.toBe(LinearFilter);
  });

  it("samples with wrap addressing, as the engine does", () => {
    // `r_Sampler` defaults to `D3DTADDRESS_WRAP` and the model blender does not override it. three.js defaults to
    // clamp, which smears the edge texel over every face whose uv leaves [0,1] - `wpn_colt1911` reaches u = -0.997.
    const texture: Nullable<CompressedTexture> = createDdsTexture(mockDdsFile());

    expect(texture!.wrapS).toBe(RepeatWrapping);
    expect(texture!.wrapT).toBe(RepeatWrapping);
  });

  it("uploads a dxt5 file", () => {
    expect(createDdsTexture(mockDdsFile({ fourCC: "DXT5" }))!.format).toBe(RGBA_S3TC_DXT5_Format);
  });

  it("refuses a bc7 file rather than uploading garbage", () => {
    // Gunslinger ships three of these. The loader logs its own complaint and returns a parse with no format.
    expect(createDdsTexture(mockDx10DdsFile(98))).toBeNull();
  });

  it("refuses an rgba ordered uncompressed file, which the loader only accepts as bgra", () => {
    // Anomaly ships 24 references to `A8B8G8R8`. Its red channel sits in the low byte, and the loader tests for red in
    // `0x00ff0000`, so it matches neither uncompressed branch.
    const bgra: ArrayBuffer = mockUncompressedDdsFile({ blueMask: 0x000000ff, redMask: 0x00ff0000 });
    const rgba: ArrayBuffer = mockUncompressedDdsFile({ blueMask: 0x00ff0000, redMask: 0x000000ff });

    expect(createDdsTexture(bgra)).not.toBeNull();
    expect(createDdsTexture(rgba)).toBeNull();
  });
});

describe("colour space", () => {
  muteConsole("error");

  it("decodes a picture from srgb and leaves packed data alone", () => {
    // A bump pair's channels are a normal rather than a colour: decoding one would bend every vector it stores, while
    // leaving a diffuse texture undecoded draws every surface brighter than the game does.
    expect(createDdsTexture(mockDdsFile(), { isColor: true })!.colorSpace).toBe(SRGBColorSpace);
    expect(createDdsTexture(mockDdsFile())!.colorSpace).toBe(NoColorSpace);
  });
});
