import { describe, expect, it } from "@jest/globals";
import { Texture } from "three";

import {
  createSurfaceMaterial,
  DEFAULT_LEVEL_SURFACE_OPTIONS,
  dressSurfaceMaterial,
  getShaderColor,
  ILevelSurface,
  ILevelSurfaceMaterial,
  ILevelSurfaceOptions,
} from "@/core/level/lib/surface/level-surface-material";
import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { IRenderDetail, OPAQUE_RENDER_SURFACE, toRenderSurface } from "@/core/render/lib/surface/render-surface";
import { mockSectorSurface } from "@/fixtures/mocks/level.mocks";
import {
  mockAlphaSurfaceDescriptor,
  mockBlendedSurfaceDescriptor,
  mockSurfaceDescriptor,
} from "@/fixtures/mocks/visual.mocks";
import { Nullable } from "@/lib/types/general";

/** A lookup answering with a distinct texture for each reference it is given. */
function lookup(...references: Array<string>): ILevelTextureLookup {
  const held: Map<string, ILevelTexture> = new Map(
    references.map((reference: string) => [
      reference,
      { isAlphaRead: false, isMipped: true, reason: null, texture: new Texture() },
    ])
  );

  return {
    listProblems: () => [],
    get: (reference: string): Nullable<ILevelTexture> => held.get(reference) ?? null,
    size: held.size,
  };
}

function surface(overrides: Partial<ILevelSurface> = {}): ILevelSurface {
  return {
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
    ? { scale: uniforms.xrayDetailScale.value as number, texture: uniforms.xrayDetail.value as Texture }
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

  // A mark is laid in the plane of the wall it marks, so the two are at one depth and the test between them is a
  // coin the camera flips on every step.
  it("pulls a composited surface towards the viewer, and leaves an opaque one where it is", () => {
    const blended: ILevelSurface = surface({ render: toRenderSurface(mockBlendedSurfaceDescriptor()) });

    expect(createSurfaceMaterial(blended, null, options()).material.polygonOffset).toBe(true);
    expect(createSurfaceMaterial(surface(), null, options()).material.polygonOffset).toBe(false);
  });

  // The second of the pair, which is what `uber_deffer` binds as `s_hemi`. The first is R1's baked colour and no
  // deferred shader samples it, so binding that one shaded every lightmapped surface by an alpha that is not
  // occlusion at all.
  it("binds the texture the renderer reads occlusion out of, and takes it off again", () => {
    const lit: ILevelSurface = surface({
      surface: mockSectorSurface({ hemi: "lmap#1_2" }),
    });
    const textures: ILevelTextureLookup = lookup("stone", "lmap#1_2");
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(lit, textures, options());

    expect(dressed.material.aoMap).toBe(textures.get("lmap#1_2")?.texture);
    expect(dressed.material.lightMap).toBeNull();

    dressSurfaceMaterial(dressed, lit, textures, options({ isLit: false }));

    expect(dressed.material.aoMap).toBeNull();
  });

  // A row the engine's own test rejects - fewer than three textures, or a third that is not spelled `lmap` - is lit
  // with no baked occlusion rather than with whatever its second texture happens to be.
  it("shades a row the renderer would not call lightmapped without any occlusion", () => {
    const unlit: ILevelSurface = surface({ surface: mockSectorSurface({ hemi: null }) });
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(unlit, lookup("stone", "lmap#1_1"), options());

    expect(dressed.material.aoMap).toBeNull();
  });

  // `v_static.color` is `(r,g,b,dir-occlusion)` and every deferred vertex shader reads the fourth component alone, so
  // the three the viewer used to multiply by are R1's baked light. Multiplying by them drew a level grey wherever
  // xrLC had baked a shadow, and the surface's own texture with it.
  it("never multiplies a surface by the vertex colour the deferred renderer ignores", () => {
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface(), lookup("stone"), options());

    expect(dressed.material.vertexColors).toBe(false);
  });

  // A checker dimmed by a night sky is just another dark surface, and the whole point of it is to be noticed.
  it("draws a surface dressed in a stand-in at full brightness", () => {
    const textures: ILevelTextureLookup = {
      get: () => ({ isAlphaRead: false, isMipped: true, reason: "placeholder", texture: new Texture() }),
      listProblems: () => [],
      size: 1,
    };
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface(), textures, options());

    expect(dressed.material.emissiveMap).toBe(dressed.material.map);
    expect(dressed.material.emissive.getHex()).toBe(0xffffff);
  });

  it("leaves a surface with its own texture lit by the scene alone", () => {
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface(), lookup("stone"), options());

    expect(dressed.material.emissiveMap).toBeNull();
    expect(dressed.material.emissive.getHex()).toBe(0x000000);
  });

  // `def_gloss` is two of two hundred and fifty five, so a level surface has no specular to speak of.
  it("draws a surface fully rough, as the gloss the g-buffer writes comes to", () => {
    expect(createSurfaceMaterial(surface(), lookup("stone"), options()).material.roughness).toBe(1);
  });

  it("dresses a surface with its base texture and clears the tint", () => {
    const textures: ILevelTextureLookup = lookup("stone");
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface(), textures, options());

    expect(dressed.material.map).toBe(textures.get("stone")?.texture);
    expect(dressed.material.color.getHex()).toBe(0xffffff);
  });

  // Always, with no switch: an untextured level in one flat white says only that it is untextured, and a surface
  // that has its texture takes its colour from it, so the two never compete.
  it("colours an untextured surface by its shader entry", () => {
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface(), null, options({ isTextured: false }));

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

  it("takes the modulation off with the textures it modulates", () => {
    const detailed: ILevelSurface = surface({ render: { ...OPAQUE_RENDER_SURFACE, detail: DETAIL } });
    const textures: ILevelTextureLookup = lookup("stone", DETAIL.reference);
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(detailed, textures, options());

    expect(applied(dressed)).not.toBeNull();

    // The modulation is of what the base texture produced and is laid out in its coordinate, so a surface drawn flat
    // has nothing for it to modulate. It has no switch of its own: it is half of what an X-Ray surface is.
    dressSurfaceMaterial(dressed, detailed, textures, options({ isTextured: false }));

    expect(applied(dressed)).toBeNull();
  });
});

describe("createSurfaceMaterial shading", () => {
  function scriptedSurface(isBlended: boolean): ILevelSurface {
    return surface({
      render: toRenderSurface(
        mockSurfaceDescriptor({
          declaration: {
            function: "normal",
            alphaReference: 0,
            isAlphaTested: true,
            isBlended,
            isDepthWritten: false,
            isWallmark: true,
            kind: "scripted",
            script: "shaders\\r2\\effects_wallmarkmult.s",
          },
          draw: { isDoubled: true, kind: "multiplied" },
        })
      ),
    });
  }

  // The regression this exists for: the material was built from the options alone, so every level surface got the
  // opaque default and the forward passes stayed lit. A wall mark multiplied into a lit wall brightens the rectangle
  // it covers instead of being neutral.
  it("builds a composited script pass unlit", () => {
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(scriptedSurface(true), null, options());

    expect(dressed.material.customProgramCacheKey()).toContain("xray-unlit");
  });

  it("leaves an ordinary level surface lit", () => {
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface(), null, options());

    expect(dressed.material.customProgramCacheKey()).not.toContain("xray-unlit");
  });

  // Built with the surface, so what its shader compiles to is on the material before it has ever drawn.
  it("carries what its shader compiles to from the moment it is built", () => {
    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(scriptedSurface(true), null, options());

    expect(dressed.material.transparent).toBe(true);
    expect(dressed.material.depthWrite).toBe(false);
  });
});
