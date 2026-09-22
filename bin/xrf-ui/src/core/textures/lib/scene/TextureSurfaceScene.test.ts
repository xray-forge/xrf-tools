import { afterEach, beforeAll, describe, expect, it, jest } from "@jest/globals";
import { mockDdsFile } from "@xrf/renderer/fixtures";
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
import {
  ETextureSurfaceAlpha,
  ETextureSurfaceShape,
  ITextureSurfaceFile,
  ITextureSurfaceFiles,
} from "@/core/textures/lib/texture-surface";

let TextureSurfaceScene: typeof import("./TextureSurfaceScene").TextureSurfaceScene;

function mockScene(): InstanceType<typeof TextureSurfaceScene> {
  const target: DomRenderTarget = new DomRenderTarget(document.createElement("div"));

  return new TextureSurfaceScene(target, target.canvas);
}

function file(fourCC: string = "DXT5"): ITextureSurfaceFile {
  return { bytes: mockDdsFile({ fourCC }), height: 8, isDecoded: false, width: 8 };
}

function files(overrides: Partial<ITextureSurfaceFiles> = {}): ITextureSurfaceFiles {
  return { aspect: 1, base: file(), bump: { bump: file(), companion: file() }, ...overrides };
}

function uploadsOf(scene: InstanceType<typeof TextureSurfaceScene>): Array<Texture> {
  return (scene as unknown as { uploaded: Array<Texture> }).uploaded;
}

function materialOf(scene: InstanceType<typeof TextureSurfaceScene>): Material {
  const mesh = (scene as unknown as { mesh: Mesh<never, Array<Material> | Material> }).mesh;

  // The flat body groups its faces and only one of them carries the texture; the others are its dark edges.
  return Array.isArray(mesh.material) ? mesh.material[4] : mesh.material;
}

describe("TextureSurfaceScene", () => {
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

  it("changes lighting without invalidating texture data or materials", () => {
    const scene = mockScene();

    try {
      scene.setTextures(files());

      const [base, bump, companion] = uploadsOf(scene);
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
    const scene = mockScene();

    try {
      scene.setTextures(files());

      const [base, bump, companion] = uploadsOf(scene);
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

  // A texture belongs to the context that made it, so this scene is the only thing that can release what it
  // uploaded - and the service that read the files no longer has any gpu memory to answer for.
  it("releases what it uploaded, for the next texture and for itself", () => {
    const scene = mockScene();

    scene.setTextures(files());

    const released: Array<Texture> = [];

    for (const texture of [...uploadsOf(scene)]) {
      texture.addEventListener("dispose", () => released.push(texture));
    }

    expect(released).toHaveLength(0);

    scene.setTextures(files());

    expect(released).toHaveLength(3);

    const second: Array<Texture> = [...uploadsOf(scene)];

    for (const texture of second) {
      texture.addEventListener("dispose", () => released.push(texture));
    }

    scene.dispose();

    expect(released).toHaveLength(6);
  });

  describe("alpha", () => {
    /**
     * @param alpha - The reading to draw the body with.
     * @param format - Format the base uploaded as, which is what decides whether there is a channel at all.
     * @returns The material state the body comes to.
     */
    function shade(alpha: ETextureSurfaceAlpha, format: PixelFormat | CompressedPixelFormat = RGBAFormat): Material {
      const scene = mockScene();

      try {
        scene.setTextures(files({ bump: null }));

        // The typings admit only an uncompressed format, though three's own loader assigns a compressed one here.
        (uploadsOf(scene)[0] as Texture).format = format as PixelFormat;
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
