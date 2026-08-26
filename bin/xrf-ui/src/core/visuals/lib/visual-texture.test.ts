import { describe, expect, it } from "@jest/globals";
import { CompressedTexture, LinearFilter, RepeatWrapping, RGB_S3TC_DXT1_Format, RGBA_S3TC_DXT5_Format } from "three";

import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import {
  createDdsTexture,
  EVisualTextureState,
  toInitialTextureState,
  toLoadableTextures,
} from "@/core/visuals/lib/visual-texture";
import { mockDdsFile, mockDx10DdsFile, mockUncompressedDdsFile } from "@/fixtures/mocks/dds.mocks";
import { mockTextureDependency } from "@/fixtures/mocks/visual.mocks";
import { muteConsole } from "@/fixtures/utils/console";
import { Nullable } from "@/lib/types/general";

describe("toLoadableTextures", () => {
  it("keeps the submeshes whose reference located a file, addressed by that file", () => {
    const textures: Array<VisualTextureDependency> = [
      mockTextureDependency({ submeshIndex: 0 }),
      mockTextureDependency({ resolution: { kind: "missing", roots: ["C:\\gamedata"] }, submeshIndex: 1 }),
      mockTextureDependency({ resolution: { kind: "noScope" }, submeshIndex: 2 }),
      mockTextureDependency({ resolution: { kind: "rejected", reason: "not a logical path" }, submeshIndex: 3 }),
    ];

    expect(toLoadableTextures(textures)).toEqual([{ logicalPath: "textures\\wpn\\wpn_ak74.dds", submeshIndex: 0 }]);
  });
});

describe("toInitialTextureState", () => {
  it("separates a texture that was not found from a reference that was never usable", () => {
    // Both end up untextured, and only one is the model's fault, so the panel must not report them the same way.
    expect(toInitialTextureState({ kind: "noScope" })).toBe(EVisualTextureState.UNRESOLVED);
    expect(toInitialTextureState({ kind: "missing", roots: ["C:\\gamedata"] })).toBe(EVisualTextureState.UNRESOLVED);
    expect(toInitialTextureState({ kind: "rejected", reason: "not a logical path" })).toBe(EVisualTextureState.FAILED);
    expect(toInitialTextureState(mockTextureDependency().resolution)).toBe(EVisualTextureState.LOADING);
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
