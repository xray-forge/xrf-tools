import { afterEach, beforeAll, describe, expect, it, jest } from "@jest/globals";
import {
  CompressedPixelFormat,
  Material,
  Mesh,
  PixelFormat,
  RGB_S3TC_DXT1_Format,
  RGBAFormat,
  Texture,
  Vector2,
} from "three";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { ETextureSurfaceAlpha, ETextureSurfaceShape } from "@/core/textures/lib/texture-surface";

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

/**
 * The material the textured face of the body is drawn with, which is the one every alpha answer lands on.
 *
 * @param scene - A mounted scene.
 * @returns Its base material.
 */
function materialOf(scene: InstanceType<typeof TextureSurfaceScene>): Material {
  const mesh = (scene as unknown as { mesh: Mesh<never, Array<Material> | Material> }).mesh;

  // The flat body groups its faces and only one of them carries the texture; the others are its dark edges.
  return Array.isArray(mesh.material) ? mesh.material[4] : mesh.material;
}

describe("TextureSurfaceScene", () => {
  it("changes lighting without invalidating texture data or materials", () => {
    const scene = new TextureSurfaceScene(new DomRenderTarget(document.createElement("div")));
    const base = new Texture();
    const bump = new Texture();
    const companion = new Texture();

    try {
      scene.setTextures({ aspect: 1, base, bump: { bump, companion } });

      const versions = [base.version, bump.version, companion.version];
      const invalidateMaterial = jest.spyOn(Material.prototype, "needsUpdate", "set");

      scene.setOptions({
        alpha: ETextureSurfaceAlpha.CUT_OUT,
        isBumped: true,
        isLit: false,
        shape: ETextureSurfaceShape.PLANE,
        tiling: 1,
      });

      expect([base.version, bump.version, companion.version]).toEqual(versions);
      expect(invalidateMaterial).not.toHaveBeenCalled();
    } finally {
      scene.dispose();
    }
  });

  it("updates texture UV matrices when tiling changes without invalidating texture data or materials", () => {
    const scene = new TextureSurfaceScene(new DomRenderTarget(document.createElement("div")));
    const base = new Texture();
    const bump = new Texture();
    const companion = new Texture();

    try {
      scene.setTextures({ aspect: 1, base, bump: { bump, companion } });

      const versions = [base.version, bump.version, companion.version];
      const invalidateMaterial = jest.spyOn(Material.prototype, "needsUpdate", "set");

      scene.setOptions({
        alpha: ETextureSurfaceAlpha.CUT_OUT,
        isBumped: true,
        isLit: true,
        shape: ETextureSurfaceShape.PLANE,
        tiling: 2,
      });

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

  describe("alpha", () => {
    /**
     * @param alpha - The reading to draw the body with.
     * @param format - Format the base uploaded as, which is what decides whether there is a channel at all.
     * @returns The material state the body comes to.
     */
    function shade(alpha: ETextureSurfaceAlpha, format: PixelFormat | CompressedPixelFormat = RGBAFormat): Material {
      const scene = new TextureSurfaceScene(new DomRenderTarget(document.createElement("div")));
      const base = new Texture();

      // The typings admit only an uncompressed format, though three's own loader assigns a compressed one here.
      base.format = format as PixelFormat;

      try {
        scene.setTextures({ aspect: 1, base, bump: null });
        scene.setOptions({ alpha, isBumped: true, isLit: true, shape: ETextureSurfaceShape.PLANE, tiling: 1 });

        return materialOf(scene);
      } finally {
        scene.dispose();
      }
    }

    // `deffer_base_flat.ps` takes three components of `tbase` and never samples the fourth.
    it("leaves the channel unread, the way the plain deferred shader does", () => {
      const material: Material = shade(ETextureSurfaceAlpha.IGNORED);

      expect(material.alphaTest).toBe(0);
      expect(material.transparent).toBe(false);
    });

    // `deffer_base_aref_flat.ps` is `clip(D.w - def_aref)` and nothing else: a hard cut, still in the opaque pass.
    it("cuts out against the engine's own reference, without compositing", () => {
      const material: Material = shade(ETextureSurfaceAlpha.CUT_OUT);

      expect(material.alphaTest).toBeCloseTo(200 / 255);
      expect(material.transparent).toBe(false);
    });

    it("composites only where a forward blender would", () => {
      const material: Material = shade(ETextureSurfaceAlpha.BLENDED);

      expect(material.alphaTest).toBe(0);
      expect(material.transparent).toBe(true);
    });

    // A channel that is not there samples as zero, so cutting against it would discard the whole body.
    it("reads nothing from a texture uploaded without a channel to read", () => {
      const material: Material = shade(ETextureSurfaceAlpha.CUT_OUT, RGB_S3TC_DXT1_Format);

      expect(material.alphaTest).toBe(0);
      expect(material.transparent).toBe(false);
    });
  });
});
