import { afterEach, describe, expect, it, jest } from "@jest/globals";
import {
  ClampToEdgeWrapping,
  LinearFilter,
  LinearMipmapLinearFilter,
  NoColorSpace,
  RepeatWrapping,
  RGBA_S3TC_DXT5_Format,
  RGBAFormat,
  SRGBColorSpace,
  Texture,
} from "three";

import { EDdsRefusal } from "@/core/render/lib/dds";
import {
  createDdsTexture,
  createDecodedTexture,
  describeTextureUpload,
  hasRenderTextureAlpha,
  IRenderTextureUpload,
  XRAY_TEXTURE_ANISOTROPY,
} from "@/core/render/lib/texture/render-texture";
import { mockDdsFile, mockDx10DdsFile, mockUncompressedDdsFile } from "@/fixtures/mocks/dds.mocks";

function uploaded(bytes: ArrayBuffer, options = {}): NonNullable<IRenderTextureUpload["texture"]> {
  const upload: IRenderTextureUpload = createDdsTexture(bytes, options);

  if (!upload.texture) {
    throw new Error(`expected an uploadable texture, got ${upload.refusal?.reason}`);
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

  // `smp_rtlinear`, which a wall mark pass binds: `D3DTADDRESS_CLAMP` and `D3DTEXF_NONE` between mips, against the
  // `smp_base` every other surface gets (`Blender_Recorder_R3.cpp`). A decal is dark marks on a neutral field, and
  // mipping one spreads the marks over the field as soon as it is minified: the footprint fills with a grey the
  // engine never draws, which is what a wall looked like it had a rectangle on.
  it("samples an unmipped texture the way the wall mark sampler does", () => {
    const texture = uploaded(mockDdsFile({ height: 8, mipmapCount: 4, width: 8 }), { isMipped: false });

    expect(texture.minFilter).toBe(LinearFilter);
    expect(texture.anisotropy).toBe(1);
    expect(texture.wrapS).toBe(ClampToEdgeWrapping);
    expect(texture.wrapT).toBe(ClampToEdgeWrapping);
  });

  it("uploads only the level an unmipped texture samples", () => {
    expect(uploaded(mockDdsFile({ height: 8, mipmapCount: 4, width: 8 }), { isMipped: false }).mipmaps).toHaveLength(1);
  });

  it("keeps the chain and the anisotropy of everything else", () => {
    const texture = uploaded(mockDdsFile({ height: 8, mipmapCount: 4, width: 8 }));

    expect(texture.mipmaps).toHaveLength(4);
    expect(texture.anisotropy).toBe(XRAY_TEXTURE_ANISOTROPY);
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

  it("hands the reader's own refusal on, rather than rewording it", () => {
    // The whole point of the result: a caller falling back to the backend matches on the category instead of on
    // prose, and still has the detail to show a person.
    const upload: IRenderTextureUpload = createDdsTexture(mockUncompressedDdsFile({ bitCount: 16, blueMask: 0x1f }));

    expect(upload.texture).toBeNull();
    expect(upload.refusal?.reason).toBe(EDdsRefusal.UNSUPPORTED_MASKS);
    expect(upload.refusal?.detail).toContain("16 bit");
  });
});

describe("hasRenderTextureAlpha", () => {
  it("has nothing to say about a texture that was never uploaded", () => {
    expect(hasRenderTextureAlpha(null)).toBe(false);
  });

  // The upload decides: a DXT1 read without alpha is `RGB_S3TC_DXT1` and has no channel to sample, whatever the file
  // stores. Everything else the reader takes carries one.
  it("reads the alpha off the format the upload settled on", () => {
    expect(hasRenderTextureAlpha(uploaded(mockDdsFile({ fourCC: "DXT1" })))).toBe(false);
    expect(hasRenderTextureAlpha(uploaded(mockDdsFile({ fourCC: "DXT1" }), { isAlphaRead: true }))).toBe(true);
    expect(hasRenderTextureAlpha(uploaded(mockDdsFile({ fourCC: "DXT5" })))).toBe(true);
    expect(hasRenderTextureAlpha(uploaded(mockUncompressedDdsFile()))).toBe(true);
  });
});

describe("describeTextureUpload", () => {
  it("says a texture was uploaded the way a wall mark samples one", () => {
    const texture: Texture = new Texture();

    texture.mipmaps = [];
    texture.minFilter = LinearFilter;
    texture.wrapS = ClampToEdgeWrapping;
    texture.anisotropy = 1;

    expect(describeTextureUpload(texture)).toBe("1 level · linear · clamped · aniso 1");
  });

  it("says a texture kept the chain and the anisotropy of an ordinary surface", () => {
    const texture: Texture = new Texture();

    texture.mipmaps = [{}, {}, {}] as never;
    texture.minFilter = LinearMipmapLinearFilter;
    texture.wrapS = RepeatWrapping;
    texture.anisotropy = 8;

    expect(describeTextureUpload(texture)).toBe("3 levels · linear between mips · wrapped · aniso 8");
  });
});
