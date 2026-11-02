import { describe, expect, it } from "@jest/globals";
import { TextureDescription } from "@xrf/ipc/types/xrf-app-textures";
import { XrayMaterialDescriptor } from "@xrf/ipc/types/xrf-material";
import { Nullable } from "@xrf/types";

import { mockTextureDescription } from "@/core/textures/fixtures/texture.mocks";
import { ITextureBumpAssets, selectTextureBumpAssets, toTextureAspect } from "@/core/textures/lib/texture-surface";
import { mockMaterialDescriptor } from "@/core/visuals/fixtures/visual.mocks";

describe("toTextureAspect", () => {
  it("should report the proportions the file was authored in", () => {
    const description: TextureDescription = mockTextureDescription(undefined, {
      base: { shape: { width: 1024, height: 2048, mipmapLevels: 12, format: "DXT1" }, size: 1024 },
    });

    expect(toTextureAspect(description)).toBe(0.5);
  });

  it("should fall back to a square when nothing measured the file", () => {
    expect(toTextureAspect(mockTextureDescription())).toBe(1);
    expect(
      toTextureAspect(
        mockTextureDescription(undefined, {
          base: { shape: { width: 64, height: 0, mipmapLevels: 1, format: "DXT1" }, size: 64 },
        })
      )
    ).toBe(1);
  });
});

describe("selectTextureBumpAssets", () => {
  it("should name both located halves of a declared pair", () => {
    const description: TextureDescription = mockTextureDescription(undefined, {
      material: mockMaterialDescriptor(),
    });
    const assets: Nullable<ITextureBumpAssets> = selectTextureBumpAssets(description);

    expect(assets?.bump.logicalPath).toBe("textures\\wpn\\wpn_ak74_bump.dds");
    expect(assets?.companion.logicalPath).toBe("textures\\wpn\\wpn_ak74_bump#.dds");
  });

  it("should name nothing for a material binding no pair", () => {
    expect(selectTextureBumpAssets(mockTextureDescription())).toBeNull();
  });

  it("should name nothing when only one half was located", () => {
    const material: XrayMaterialDescriptor = mockMaterialDescriptor();
    const description: TextureDescription = mockTextureDescription(undefined, {
      material: {
        ...material,
        bump: material.bump && {
          ...material.bump,
          // A pair the engine would take the bump path for and shade with its flat dummy: half of it shades nothing,
          // so the surface draws neither rather than a half-decoded one.
          companion: {
            ...material.bump.companion,
            resolution: { kind: "missing", roots: ["C:\\gamedata"] },
          },
        },
      },
    });

    expect(selectTextureBumpAssets(description)).toBeNull();
  });
});
