import { describe, expect, it } from "@jest/globals";

import { ETextureSurfaceAlpha, ETextureSurfaceShape } from "@/core/ipc/types/xrf-app";
import { RenderViewOptions } from "@/core/ipc/types/xrf-renderer";
import { toTextureSurfaceRequest, toTextureViewOptions } from "@/core/textures/lib/render/texture-surface-render";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/texture-lighting";
import { ITextureSurfaceOptions } from "@/core/textures/lib/texture-surface";
import { VIEWPORT } from "@/core/theme/tokens";
import { mockRenderFeatures } from "@/fixtures/mocks/render.mocks";
import { MOCK_TEXTURE, mockTextureDescription } from "@/fixtures/mocks/texture.mocks";

const OPTIONS: ITextureSurfaceOptions = {
  alpha: ETextureSurfaceAlpha.BLENDED,
  isBumped: false,
  isLit: false,
  shape: ETextureSurfaceShape.CUBE,
  tiling: 3,
};

describe("toTextureSurfaceRequest", () => {
  it("asks for the texture as its description named it, on the body and with the alpha the view asks", () => {
    const description = mockTextureDescription(MOCK_TEXTURE, {
      base: { shape: { format: "DXT1", height: 512, mipmapLevels: 1, width: 1024 }, size: 1 },
    });

    expect(toTextureSurfaceRequest(description, OPTIONS)).toEqual({
      alpha: ETextureSurfaceAlpha.BLENDED,
      aspect: 2,
      roots: description.roots,
      shape: ETextureSurfaceShape.CUBE,
      source: description.source,
      tiling: 3,
    });
  });

  it("asks for nothing while no texture is open", () => {
    expect(toTextureSurfaceRequest(null, OPTIONS)).toBeNull();
  });
});

describe("toTextureViewOptions", () => {
  it("carries the lit and bump switches, against the alpha checkerboard at the device's pixel size", () => {
    const options: RenderViewOptions = toTextureViewOptions(OPTIONS, DEFAULT_TEXTURE_LIGHTING, mockRenderFeatures(), 2);

    expect(options.isLit).toBe(false);
    expect(options.isBumped).toBe(false);
    expect(options.backdropSquares?.size).toBe(VIEWPORT.checkerboardSquare * 2);
    expect(options.isSkyVisible).toBe(false);
    expect(options.exposure.isEnabled).toBe(false);
  });
});
