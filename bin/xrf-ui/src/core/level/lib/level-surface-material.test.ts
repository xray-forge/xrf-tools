import { describe, expect, it } from "@jest/globals";
import { MeshStandardMaterial, Texture } from "three";

import {
  createSurfaceMaterial,
  DEFAULT_LEVEL_SURFACE_OPTIONS,
  dressSurfaceMaterial,
  getShaderColor,
  ILevelSurface,
  ILevelSurfaceOptions,
} from "@/core/level/lib/level-surface-material";
import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/level-texture-set";
import { OPAQUE_RENDER_SURFACE, toRenderSurface } from "@/core/render/lib/render-surface";
import { mockSectorSurface } from "@/fixtures/mocks/level.mocks";
import { mockAlphaSurfaceDescriptor } from "@/fixtures/mocks/visual.mocks";
import { Nullable } from "@/lib/types/general";

/** A lookup answering with a distinct texture for each reference it is given. */
function lookup(...references: Array<string>): ILevelTextureLookup {
  const held: Map<string, ILevelTexture> = new Map(
    references.map((reference: string) => [reference, { reason: null, texture: new Texture() }])
  );

  return {
    get: (reference: string): Nullable<ILevelTexture> => held.get(reference) ?? null,
    size: held.size,
  };
}

function surface(overrides: Partial<ILevelSurface> = {}): ILevelSurface {
  return {
    hasVertexColors: false,
    render: OPAQUE_RENDER_SURFACE,
    surface: mockSectorSurface(),
    ...overrides,
  };
}

function options(overrides: Partial<ILevelSurfaceOptions> = {}): ILevelSurfaceOptions {
  return { ...DEFAULT_LEVEL_SURFACE_OPTIONS, ...overrides };
}

describe("level surface material", () => {
  // The defect the whole surface pass exists to fix: a marsh's reeds name `B_TREE`, and drawing them without their
  // blender's answer puts a field of solid black cards where the foliage should be.
  it("cuts out a surface whose blender reads alpha", () => {
    const cutOut: ILevelSurface = surface({ render: toRenderSurface(mockAlphaSurfaceDescriptor()) });
    const material: MeshStandardMaterial = createSurfaceMaterial(cutOut, null, options());

    expect(material.alphaTest).toBeCloseTo(200 / 255);
    expect(material.transparent).toBe(false);
    expect(material.depthWrite).toBe(true);
  });

  it("draws an alpha surface solid while the comparison is on", () => {
    const cutOut: ILevelSurface = surface({ render: toRenderSurface(mockAlphaSurfaceDescriptor()) });
    const material: MeshStandardMaterial = createSurfaceMaterial(cutOut, null, options({ isAlphaVisible: false }));

    expect(material.alphaTest).toBe(0);
  });

  it("binds a lightmap and the baked vertex colour, and takes both off together", () => {
    const lit: ILevelSurface = surface({
      hasVertexColors: true,
      surface: mockSectorSurface({ lightmaps: ["lmap#1_1", "lmap#1_2"] }),
    });
    const textures: ILevelTextureLookup = lookup("stone", "lmap#1_1");
    const material: MeshStandardMaterial = createSurfaceMaterial(lit, textures, options());

    // The first of the pair only: xrLC writes two and this samples one, which is the approximation this viewer makes.
    expect(material.lightMap).toBe(textures.get("lmap#1_1")?.texture);
    expect(material.vertexColors).toBe(true);

    dressSurfaceMaterial(material, lit, textures, options({ isLit: false }));

    expect(material.lightMap).toBeNull();
    expect(material.vertexColors).toBe(false);
  });

  it("never switches vertex colour on for geometry that carries none", () => {
    // Switching it on without the attribute leaves the shader reading a buffer that is not bound, which draws
    // nothing at all rather than drawing the surface unlit.
    const material: MeshStandardMaterial = createSurfaceMaterial(surface(), lookup("stone"), options());

    expect(material.vertexColors).toBe(false);
  });

  it("dresses a surface with its base texture and clears the tint", () => {
    const textures: ILevelTextureLookup = lookup("stone");
    const material: MeshStandardMaterial = createSurfaceMaterial(surface(), textures, options());

    expect(material.map).toBe(textures.get("stone")?.texture);
    expect(material.color.getHex()).toBe(0xffffff);
  });

  it("colours an untextured surface by its shader entry when asked", () => {
    const material: MeshStandardMaterial = createSurfaceMaterial(
      surface(),
      null,
      options({ isSurfaceColored: true, isTextured: false })
    );

    expect(material.color.getHex()).toBe(getShaderColor(mockSectorSurface().shaderId).getHex());
  });

  it("gives one shader entry the same colour wherever it is drawn", () => {
    expect(getShaderColor(7).getHex()).toBe(getShaderColor(7).getHex());
    expect(getShaderColor(7).getHex()).not.toBe(getShaderColor(8).getHex());
  });
});
