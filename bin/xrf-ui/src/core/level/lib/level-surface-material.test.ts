import { describe, expect, it } from "@jest/globals";
import { Texture, Vector2 } from "three";

import {
  createSurfaceMaterial,
  DEFAULT_LEVEL_SURFACE_OPTIONS,
  dressSurfaceMaterial,
  getShaderColor,
  ILevelSurface,
  ILevelSurfaceMaterial,
  ILevelSurfaceOptions,
} from "@/core/level/lib/level-surface-material";
import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/level-texture-set";
import { IRenderDetail, OPAQUE_RENDER_SURFACE, toRenderSurface } from "@/core/render/lib/render-surface";
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

const DETAIL: IRenderDetail = { reference: "detail\\detail_grnd_earth", scale: 150 };

function applied(dressed: ILevelSurfaceMaterial): Nullable<{ scale: number; texture: Texture }> {
  const shader = { fragmentShader: "#include <map_fragment>", uniforms: {}, vertexShader: "" };

  dressed.material.onBeforeCompile(shader as never, null as never);

  const uniforms = shader.uniforms as Record<string, { value: unknown }>;

  return uniforms.xrayDetailEnabled?.value
    ? { scale: (uniforms.xrayDetailParams.value as Vector2).x, texture: uniforms.xrayDetail.value as Texture }
    : null;
}

describe("level surface material", () => {
  // The defect the whole surface pass exists to fix: a marsh's reeds name `B_TREE`, and drawing them without their
  // blender's answer puts a field of solid black cards where the foliage should be.
  it("cuts out a surface whose blender reads alpha", () => {
    const cutOut: ILevelSurface = surface({ render: toRenderSurface(mockAlphaSurfaceDescriptor()) });
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(cutOut, null, options());

    expect(dressed.material.alphaTest).toBeCloseTo(200 / 255);
    expect(dressed.material.transparent).toBe(false);
    expect(dressed.material.depthWrite).toBe(true);
  });

  it("draws an alpha surface solid while the comparison is on", () => {
    const cutOut: ILevelSurface = surface({ render: toRenderSurface(mockAlphaSurfaceDescriptor()) });
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(cutOut, null, options({ isAlphaVisible: false }));

    expect(dressed.material.alphaTest).toBe(0);
  });

  it("binds a lightmap and the baked vertex colour, and takes both off together", () => {
    const lit: ILevelSurface = surface({
      hasVertexColors: true,
      surface: mockSectorSurface({ lightmaps: ["lmap#1_1", "lmap#1_2"] }),
    });
    const textures: ILevelTextureLookup = lookup("stone", "lmap#1_1");
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(lit, textures, options());

    // The first of the pair only: xrLC writes two and this samples one, which is the approximation this viewer makes.
    expect(dressed.material.lightMap).toBe(textures.get("lmap#1_1")?.texture);
    expect(dressed.material.vertexColors).toBe(true);

    dressSurfaceMaterial(dressed, lit, textures, options({ isLit: false }));

    expect(dressed.material.lightMap).toBeNull();
    expect(dressed.material.vertexColors).toBe(false);
  });

  it("never switches vertex colour on for geometry that carries none", () => {
    // Switching it on without the attribute leaves the shader reading a buffer that is not bound, which draws
    // nothing at all rather than drawing the surface unlit.
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface(), lookup("stone"), options());

    expect(dressed.material.vertexColors).toBe(false);
  });

  it("dresses a surface with its base texture and clears the tint", () => {
    const textures: ILevelTextureLookup = lookup("stone");
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface(), textures, options());

    expect(dressed.material.map).toBe(textures.get("stone")?.texture);
    expect(dressed.material.color.getHex()).toBe(0xffffff);
  });

  it("colours an untextured surface by its shader entry when asked", () => {
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(
      surface(),
      null,
      options({ isSurfaceColored: true, isTextured: false })
    );

    expect(dressed.material.color.getHex()).toBe(getShaderColor(mockSectorSurface().shaderId).getHex());
  });

  it("gives one shader entry the same colour wherever it is drawn", () => {
    expect(getShaderColor(7).getHex()).toBe(getShaderColor(7).getHex());
    expect(getShaderColor(7).getHex()).not.toBe(getShaderColor(8).getHex());
  });

  // The defect the detail pass exists to fix: a terrain's base texture covers the whole level at a resolution no
  // close camera survives, and without the tiled texture the engine multiplies over it the ground is a grey blur.
  it("modulates a detailed surface with the texture and the tiling its blender resolved", () => {
    const detailed: ILevelSurface = surface({ render: { ...OPAQUE_RENDER_SURFACE, detail: DETAIL } });
    const textures: ILevelTextureLookup = lookup("stone", DETAIL.reference);
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(detailed, textures, options());

    expect(dressed.detail).not.toBeNull();
    expect(applied(dressed)).toEqual({ scale: 150, texture: textures.get(DETAIL.reference)?.texture });
  });

  it("compiles no modulation into a surface nothing details", () => {
    // The patch is what a detailed surface costs, and a level is mostly surfaces that are not.
    expect(createSurfaceMaterial(surface(), lookup("stone"), options()).detail).toBeNull();
  });

  it("draws a detailed surface undetailed until its texture has been read", () => {
    const detailed: ILevelSurface = surface({ render: { ...OPAQUE_RENDER_SURFACE, detail: DETAIL } });
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(detailed, lookup("stone"), options());

    expect(applied(dressed)).toBeNull();
  });

  it("takes the modulation off with its own toggle and with the textures it modulates", () => {
    const detailed: ILevelSurface = surface({ render: { ...OPAQUE_RENDER_SURFACE, detail: DETAIL } });
    const textures: ILevelTextureLookup = lookup("stone", DETAIL.reference);
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(detailed, textures, options());

    dressSurfaceMaterial(dressed, detailed, textures, options({ isDetailed: false }));

    expect(applied(dressed)).toBeNull();

    // And with the base texture, since the modulation is of what the base texture produced and is laid out in its
    // coordinate: a surface drawn flat has nothing for it to modulate.
    dressSurfaceMaterial(dressed, detailed, textures, options({ isTextured: false }));

    expect(applied(dressed)).toBeNull();
  });
});
