import { afterEach, describe, expect, it, jest } from "@jest/globals";
import {
  LinearFilter,
  NoColorSpace,
  RepeatWrapping,
  RGBA_S3TC_DXT5_Format,
  RGBAFormat,
  SRGBColorSpace,
  Texture,
} from "three";

import { createDdsTexture, createDecodedTexture, IRenderTextureUpload } from "@/core/render/lib/render-texture";
import { mockDdsFile, mockDx10DdsFile, mockUncompressedDdsFile } from "@/fixtures/mocks/dds.mocks";

function uploaded(bytes: ArrayBuffer, options = {}): NonNullable<IRenderTextureUpload["texture"]> {
  const upload: IRenderTextureUpload = createDdsTexture(bytes, options);

  if (!upload.texture) {
    throw new Error(`expected an uploadable texture, got ${upload.refusal}`);
  }

  return upload.texture;
}

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
  it("uploads a block layout in the format the reader read", () => {
    const texture = uploaded(mockDdsFile({ fourCC: "DXT5", height: 8, mipmapCount: 2, width: 8 }));

    expect(texture.format).toBe(RGBA_S3TC_DXT5_Format);
    expect(texture.image.width).toBe(8);
    expect(texture.mipmaps).toHaveLength(2);
  });

  it("uploads an expanded layout as rgba, the way three's own loader does", () => {
    expect(uploaded(mockUncompressedDdsFile()).format).toBe(RGBAFormat as never);
  });

  it("drops to a non mipmap filter when the file carries no mip chain", () => {
    // Load bearing rather than cosmetic: webgl samples an incomplete texture as black, and most modded textures ship
    // without mips - 1,805 of Anomaly's 2,197.
    expect(uploaded(mockDdsFile({ mipmapCount: 1 })).minFilter).toBe(LinearFilter);
    expect(uploaded(mockDdsFile({ height: 8, mipmapCount: 4, width: 8 })).minFilter).not.toBe(LinearFilter);
  });

  it("samples with wrap addressing, as the engine does", () => {
    // `r_Sampler` defaults to `D3DTADDRESS_WRAP` and the model blender does not override it. three.js defaults to
    // clamp, which smears the edge texel over every face whose uv leaves [0,1] - `wpn_colt1911` reaches u = -0.997.
    const texture = uploaded(mockDdsFile());

    expect(texture.wrapS).toBe(RepeatWrapping);
    expect(texture.wrapT).toBe(RepeatWrapping);
  });

  it("decodes a picture from srgb and leaves packed data alone", () => {
    // A bump pair's channels are a normal rather than a colour: decoding one would bend every vector it stores, while
    // leaving a diffuse texture undecoded draws every surface brighter than the game does.
    expect(uploaded(mockDdsFile(), { isColor: true }).colorSpace).toBe(SRGBColorSpace);
    expect(uploaded(mockDdsFile()).colorSpace).toBe(NoColorSpace);
  });

  it("uploads what the example loader used to refuse", () => {
    // BC3 under a DX10 header, and BC7. Both are formats three renders natively and the old path sent to the backend
    // to be decoded into a mipless png.
    expect(uploaded(mockDx10DdsFile(77)).format).toBe(RGBA_S3TC_DXT5_Format);
    expect(createDdsTexture(mockDx10DdsFile(98)).texture).not.toBeNull();
  });

  it("names the layout it refused rather than answering with a bare nothing", () => {
    // The whole point of the result: a caller falling back to the backend can now say which layout put it there.
    const upload: IRenderTextureUpload = createDdsTexture(mockUncompressedDdsFile({ bitCount: 16, blueMask: 0x1f }));

    expect(upload.texture).toBeNull();
    expect(upload.refusal).toContain("unsupportedMasks");
  });

  it("refuses a cubemap rather than drawing one face stretched over a surface", () => {
    const bytes: ArrayBuffer = mockDdsFile();

    // `DDSCAPS2_CUBEMAP` and its six faces.
    new Uint32Array(bytes)[28] = 0x200 | 0x400 | 0x800 | 0x1000 | 0x2000 | 0x4000 | 0x8000;

    expect(createDdsTexture(bytes).texture).toBeNull();
  });
});
