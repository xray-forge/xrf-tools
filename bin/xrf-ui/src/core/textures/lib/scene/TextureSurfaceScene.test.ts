import { afterEach, beforeAll, describe, expect, it, jest } from "@jest/globals";
import { Material, Texture, Vector2 } from "three";

import { ETextureSurfaceShape } from "@/core/textures/lib/texture-surface";

let TextureSurfaceScene: typeof import("./TextureSurfaceScene").TextureSurfaceScene;

beforeAll(async () => {
  const three = jest.requireActual<typeof import("three")>("three");

  // Keep real textures, materials and controls; only the GPU boundary is unavailable in jsdom.
  jest.doMock("three", () => ({
    ...three,
    WebGLRenderer: jest.fn(() => ({
      domElement: document.createElement("canvas"),
      setPixelRatio: jest.fn(),
      dispose: jest.fn(),
      forceContextLoss: jest.fn(),
    })),
  }));

  ({ TextureSurfaceScene } = await import("./TextureSurfaceScene"));
});

afterEach(() => {
  jest.restoreAllMocks();
});

describe("TextureSurfaceScene", () => {
  it("changes lighting without invalidating texture data or materials", () => {
    const scene = new TextureSurfaceScene();
    const base = new Texture();
    const bump = new Texture();
    const companion = new Texture();

    try {
      scene.setTextures({ aspect: 1, base, bump: { bump, companion } });

      const versions = [base.version, bump.version, companion.version];
      const invalidateMaterial = jest.spyOn(Material.prototype, "needsUpdate", "set");

      scene.setOptions({ isBumped: true, isLit: false, shape: ETextureSurfaceShape.PLANE, tiling: 1 });

      expect([base.version, bump.version, companion.version]).toEqual(versions);
      expect(invalidateMaterial).not.toHaveBeenCalled();
    } finally {
      scene.dispose();
    }
  });

  it("updates texture UV matrices when tiling changes without invalidating texture data or materials", () => {
    const scene = new TextureSurfaceScene();
    const base = new Texture();
    const bump = new Texture();
    const companion = new Texture();

    try {
      scene.setTextures({ aspect: 1, base, bump: { bump, companion } });

      const versions = [base.version, bump.version, companion.version];
      const invalidateMaterial = jest.spyOn(Material.prototype, "needsUpdate", "set");

      scene.setOptions({ isBumped: true, isLit: true, shape: ETextureSurfaceShape.PLANE, tiling: 2 });

      for (const texture of [base, bump, companion]) {
        expect(texture.repeat.toArray()).toEqual([2, 2]);
        expect(new Vector2(0.25, 0.5).applyMatrix3(texture.matrix).toArray()).toEqual([0.5, 1]);
      }

      expect([base.version, bump.version, companion.version]).toEqual(versions);
      expect(invalidateMaterial).not.toHaveBeenCalled();
    } finally {
      scene.dispose();
    }
  });
});
