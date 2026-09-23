import { describe, expect, it } from "@jest/globals";
import { ERendererDraw, ERendererTextureEncoding, IRendererGeometry, IRendererObject } from "@xrf/renderer";

import {
  createTextureSurfaceGeometry,
  TEXTURE_SURFACE_KEYS,
  toTextureRendererSettings,
  toTextureSurface,
  toTextureSurfaceObject,
  toTextureSurfaceSource,
} from "@/core/textures/lib/render/texture-surface-render";
import { DEFAULT_TEXTURE_PREVIEW_OPTIONS } from "@/core/textures/lib/texture-preview";
import {
  EMPTY_TEXTURE_SURFACE,
  ETextureSurfaceAlpha,
  ETextureSurfaceShape,
  ITextureSurfaceFile,
} from "@/core/textures/lib/texture-surface";

const FILE: ITextureSurfaceFile = { bytes: new ArrayBuffer(8), height: 2, isDecoded: false, width: 2 };

describe("toTextureSurfaceObject", () => {
  // Every group a body draws needs a surface, or the renderer hides it: a cube given one surface showed one face.
  it.each(Object.values(ETextureSurfaceShape))("names a surface for every face the %s draws", (shape) => {
    const geometry: IRendererGeometry = createTextureSurfaceGeometry(shape);
    const object: IRendererObject = toTextureSurfaceObject(shape, 1);

    expect(object.surfaces).toHaveLength(geometry.groups.length);
  });

  it("textures every face of the cube and only the front of the slab", () => {
    const { edge, face } = TEXTURE_SURFACE_KEYS;

    expect(toTextureSurfaceObject(ETextureSurfaceShape.CUBE, 1).surfaces).toEqual(Array(6).fill(face));
    expect(toTextureSurfaceObject(ETextureSurfaceShape.PLANE, 1).surfaces).toEqual([
      edge,
      edge,
      edge,
      edge,
      face,
      edge,
    ]);
  });

  it("stretches only the slab to the texture's proportions", () => {
    expect(toTextureSurfaceObject(ETextureSurfaceShape.PLANE, 2).matrix?.slice(0, 6)).toEqual([1, 0, 0, 0, 0, 0.5]);
    expect(toTextureSurfaceObject(ETextureSurfaceShape.CUBE, 2).matrix).toBeUndefined();
  });
});

describe("toTextureSurface", () => {
  it("draws each alpha reading as the engine's draw", () => {
    function draw(alpha: ETextureSurfaceAlpha): ERendererDraw {
      return toTextureSurface(EMPTY_TEXTURE_SURFACE, { ...DEFAULT_TEXTURE_PREVIEW_OPTIONS, alpha }).draw;
    }

    expect(draw(ETextureSurfaceAlpha.IGNORED)).toBe(ERendererDraw.OPAQUE);
    expect(draw(ETextureSurfaceAlpha.CUT_OUT)).toBe(ERendererDraw.CUT_OUT);
    expect(draw(ETextureSurfaceAlpha.BLENDED)).toBe(ERendererDraw.BLENDED);
  });

  it("names the pair only when both halves were read", () => {
    const textures = toTextureSurface(
      { ...EMPTY_TEXTURE_SURFACE, base: FILE, bump: { bump: FILE, companion: FILE } },
      DEFAULT_TEXTURE_PREVIEW_OPTIONS
    ).textures;

    expect(textures).toEqual({ base: "base", bump: "bump", bumpCompanion: "bump#" });
    expect(toTextureSurface(EMPTY_TEXTURE_SURFACE, DEFAULT_TEXTURE_PREVIEW_OPTIONS).textures.bump).toBeUndefined();
  });
});

describe("toTextureSurfaceSource", () => {
  it("copies the bytes, so the transfer leaves the surface's own", () => {
    const source = toTextureSurfaceSource(FILE);

    expect(source.bytes).not.toBe(FILE.bytes);
    expect(source.encoding).toBe(ERendererTextureEncoding.DDS);
    expect(toTextureSurfaceSource({ ...FILE, isDecoded: true }).encoding).toBe(ERendererTextureEncoding.IMAGE);
  });
});

describe("toTextureRendererSettings", () => {
  it("carries the lit and bump switches, over a transparent backdrop", () => {
    const settings = toTextureRendererSettings(
      { ...DEFAULT_TEXTURE_PREVIEW_OPTIONS, isBumped: false, isLit: false },
      "60"
    );

    expect(settings.backdrop).toBeNull();
    expect(settings.isLit).toBe(false);
    expect(settings.isBumped).toBe(false);
  });
});
