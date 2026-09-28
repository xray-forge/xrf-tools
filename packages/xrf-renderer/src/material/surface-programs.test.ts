import { describe, expect, it } from "@jest/globals";
import { MeshBasicNodeMaterial } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { ERendererPass } from "#/contract/scene/renderer-pass";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { createOpaqueShadowMaterial, createSurfaceMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { ESurfaceSlot, getSurfaceSlotPlaceholder } from "#/material/surface-slot";
import { toSampledSlots, toSurfaceVariant, toSurfaceVariantKey } from "#/material/surface-variant";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

const BRICK: IRendererSurface = {
  alphaReference: 0.5,
  detailScale: 4,
  draw: ERendererDraw.CUT_OUT,
  material: 2,
  textures: { base: "brick", detail: "detail\\stone", hemi: "lmap#1" },
  tiling: 2,
};

/** The same kind of surface as the brick: other textures, other numbers. */
const PLASTER: IRendererSurface = {
  alphaReference: 0.25,
  detailScale: 8,
  draw: ERendererDraw.CUT_OUT,
  material: 1,
  textures: { base: "plaster", detail: "detail\\plaster", hemi: "lmap#2" },
  tiling: 3,
};

function createTextures(): RendererTextures {
  return new RendererTextures(
    () => {},
    () => {}
  );
}

function createMaterial(
  surface: IRendererSurface,
  programs: SurfacePrograms,
  uniforms: RendererUniforms,
  textures: RendererTextures = createTextures()
): ISurfaceMaterial {
  const opaque: MeshBasicNodeMaterial = createOpaqueShadowMaterial(programs, uniforms);

  return createSurfaceMaterial(surface, textures, uniforms, programs, opaque);
}

describe("surface variants", () => {
  it("are equal for surfaces differing only in their textures and numbers", () => {
    expect(toSurfaceVariantKey(toSurfaceVariant(BRICK))).toBe(toSurfaceVariantKey(toSurfaceVariant(PLASTER)));
  });

  it("differ where the shader's shape does: a slot sampled, a tint, the draw", () => {
    const key: string = toSurfaceVariantKey(toSurfaceVariant(BRICK));

    expect(toSurfaceVariantKey(toSurfaceVariant({ ...BRICK, textures: { base: "brick" } }))).not.toBe(key);
    expect(toSurfaceVariantKey(toSurfaceVariant({ ...BRICK, color: [1, 0, 0] }))).not.toBe(key);
    expect(toSurfaceVariantKey(toSurfaceVariant({ ...BRICK, draw: ERendererDraw.OPAQUE }))).not.toBe(key);
  });

  it("sample the slots their shader reads and no others", () => {
    expect(toSampledSlots(toSurfaceVariant(BRICK))).toEqual([
      ESurfaceSlot.BASE,
      ESurfaceSlot.DETAIL,
      ESurfaceSlot.HEMI,
    ]);
    // Half a bump pair is sampled by nothing.
    expect(toSampledSlots(toSurfaceVariant({ ...BRICK, textures: { base: "brick", bump: "brick_bump" } }))).toEqual([
      ESurfaceSlot.BASE,
    ]);
  });
});

describe("SurfacePrograms", () => {
  it("builds one shader a variant, which every material of it draws with", () => {
    const uniforms: RendererUniforms = new RendererUniforms();
    const programs: SurfacePrograms = new SurfacePrograms(uniforms);
    const brick: ISurfaceMaterial = createMaterial(BRICK, programs, uniforms);
    const plaster: ISurfaceMaterial = createMaterial(PLASTER, programs, uniforms);

    expect(plaster.material.fragmentNode).toBe(brick.material.fragmentNode);
    expect(plaster.material.positionNode).toBe(brick.material.positionNode);
    // Three builds a node graph once for every program key it has not seen.
    expect(plaster.material.customProgramCacheKey()).toBe(brick.material.customProgramCacheKey());
    expect(plaster.shadow?.customProgramCacheKey()).toBe(brick.shadow?.customProgramCacheKey());
  });

  it("gives each material its own textures and numbers, and its cut-out shadow the same ones", () => {
    const uniforms: RendererUniforms = new RendererUniforms();
    const programs: SurfacePrograms = new SurfacePrograms(uniforms);
    const brick: ISurfaceMaterial = createMaterial(BRICK, programs, uniforms);
    const material: SurfaceNodeMaterial = brick.material as SurfaceNodeMaterial;
    const shadow: SurfaceNodeMaterial = brick.shadow as SurfaceNodeMaterial;

    expect(material.surfaceValues).toMatchObject({ alphaReference: 0.5, detailScale: 4, tiling: 2 });
    expect(material.surfaceValues?.slice).toBeCloseTo(2.5 / 4);
    expect(shadow.surfaceSlots).toBe(material.surfaceSlots);
    expect(shadow.surfaceValues).toBe(material.surfaceValues);
    // Nothing is on the GPU yet: every slot draws its placeholder.
    expect(material.surfaceSlots?.[ESurfaceSlot.DETAIL].value).toBe(getSurfaceSlotPlaceholder(ESurfaceSlot.DETAIL));
    expect(brick.keys).toEqual(["brick", "detail\\stone", "lmap#1"]);
    expect(brick.shadowKeys).toEqual(["brick"]);
  });

  it("lets go of a material's textures when it goes", () => {
    const uniforms: RendererUniforms = new RendererUniforms();
    const programs: SurfacePrograms = new SurfacePrograms(uniforms);
    const textures: RendererTextures = createTextures();
    const brick: ISurfaceMaterial = createMaterial(BRICK, programs, uniforms, textures);

    brick.dispose();

    // A key nothing holds and nothing draws is forgotten: it counts as uploaded.
    expect(textures.isUploaded("brick")).toBe(true);
  });
});

/** `effects\water` as a level puts it: the script's own textures, soft, by OpenXRay's model. */
const WATER: IRendererSurface = {
  draw: ERendererDraw.WATER,
  textures: {
    base: "water\\water_water",
    distortion: "water\\water_dudv",
    foam: "water\\water_foam",
    normal: "water\\water_normal",
  },
  water: { anomaly: null, isSoft: true },
};

describe("water surfaces", () => {
  it("are drawn by the water pass, sampling their four slots and waiting for them", () => {
    const uniforms: RendererUniforms = new RendererUniforms();
    const water: ISurfaceMaterial = createMaterial(WATER, new SurfacePrograms(uniforms), uniforms);

    expect(water.pass).toBe(ERendererPass.WATER);
    expect(water.shadow).toBeNull();
    expect(toSampledSlots(toSurfaceVariant(WATER))).toEqual([
      ESurfaceSlot.BASE,
      ESurfaceSlot.NORMAL,
      ESurfaceSlot.FOAM,
      ESurfaceSlot.DISTORTION,
    ]);
    expect(water.keys).toEqual(["water\\water_water", "water\\water_normal", "water\\water_foam", "water\\water_dudv"]);
    // Tested against the depth and never pulled towards the eye: water lies where it lies.
    expect(water.material.depthWrite).toBe(false);
    expect(water.material.polygonOffset).toBe(false);
  });

  it("share a shader but by how soft they are and by which model draws them", () => {
    const key: string = toSurfaceVariantKey(toSurfaceVariant(WATER));
    const anomaly = { isFoamed: false, isReflecting: true, isSpecular: true, isTransparent: false };

    expect(toSurfaceVariantKey(toSurfaceVariant({ ...WATER, textures: { base: "water\\water_studen" } }))).toBe(key);
    expect(toSurfaceVariantKey(toSurfaceVariant({ ...WATER, water: { anomaly: null, isSoft: false } }))).not.toBe(key);
    expect(toSurfaceVariantKey(toSurfaceVariant({ ...WATER, water: { anomaly, isSoft: true } }))).not.toBe(key);
    expect(
      toSurfaceVariantKey(
        toSurfaceVariant({ ...WATER, water: { anomaly: { ...anomaly, isFoamed: true }, isSoft: true } })
      )
    ).not.toBe(toSurfaceVariantKey(toSurfaceVariant({ ...WATER, water: { anomaly, isSoft: true } })));
  });
});
