import { Nullable } from "@xrf/types";
import { DoubleSide, MeshBasicNodeMaterial } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { ERendererPass } from "#/contract/scene/renderer-pass";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TSurfaceArrayTargets } from "#/material/surface-array-targets";
import { ISurfaceBatchMaterial } from "#/material/surface-batch-material";
import { applySurfaceCompositing, ISurfaceCompositing, toSurfaceCompositing } from "#/material/surface-compositing";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { ISurfacePlain } from "#/material/surface-plain";
import { SurfacePrograms } from "#/material/surface-programs";
import { ISurfaceShader } from "#/material/surface-shader";
import { ESurfaceSlot } from "#/material/surface-slot";
import { TSurfaceSlotTargets } from "#/material/surface-slot-targets";
import { SurfaceSlots } from "#/material/surface-slots";
import { ISurfaceValues, toSurfaceValues } from "#/material/surface-values";
import { ISurfaceVariant, toSampledSlots, toSurfaceVariant } from "#/material/surface-variant";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { SURFACE_NO_ROW } from "#/uniforms/surface-table";

/**
 * A surface as the frame draws it.
 */
export interface ISurfaceMaterial {
  material: MeshBasicNodeMaterial;
  /** What a part of it drawn plainly draws with: its material and shadow, or for a batched view the surface's own. */
  plain: ISurfacePlain;
  /**
   * Its view in a static batch drawn by a material its surfaces share, or null while it draws static batches by its own
   * material: its array slots' textures not up yet, or of no class an array holds.
   */
  batched: Nullable<ISurfaceMaterial>;
  /** The surface table's row a shared material reads for it, `SURFACE_NO_ROW` for one drawing by its own. */
  row: number;
  /** Which pass draws it. */
  pass: ERendererPass;
  /** The texture keys it samples, which have to be uploaded before it draws without a stall. */
  keys: ReadonlyArray<string>;
  /** Whether it draws impostors, which only the LOD cull decides are drawn. */
  isImpostor: boolean;
  /**
   * What draws it into the sun's shadow maps, its depth alone; null for a surface that casts none: anything the
   * G-buffer does not draw, and an impostor, since the shadow phase casts every clump as its trees.
   */
  shadow: Nullable<MeshBasicNodeMaterial>;
  /** The texture keys its shadow material samples: a cut-out's base, and none for the one every opaque surface shares. */
  shadowKeys: ReadonlyArray<string>;
  dispose(): void;
}

/**
 * @param surface - What the consumer put.
 * @param textures - Where its textures are bound from.
 * @param uniforms - What the frame's shaders read.
 * @param programs - The shaders its variant shares.
 * @param opaqueShadow - The shadow material every opaque surface shares, its maker's.
 * @returns The material, shaded by the pass its draw puts it in.
 */
export function createSurfaceMaterial(
  surface: IRendererSurface,
  textures: RendererTextures,
  uniforms: RendererUniforms,
  programs: SurfacePrograms,
  opaqueShadow: MeshBasicNodeMaterial
): ISurfaceMaterial {
  const variant: ISurfaceVariant = toSurfaceVariant(surface);
  const shader: ISurfaceShader = programs.get(variant);
  const slots: SurfaceSlots = new SurfaceSlots(textures, surface.textures, toSampledSlots(variant));
  const values: ISurfaceValues = toSurfaceValues(surface);
  const compositing: Nullable<ISurfaceCompositing> = toSurfaceCompositing(surface);
  const material: SurfaceNodeMaterial = createSharedMaterial(programs, uniforms, slots.targets, values);

  applySurfaceShader(material, shader);

  if (variant.isImpostor) {
    // Its quad turns to face the camera, from whichever side it is seen.
    material.side = DoubleSide;
  }

  if (compositing) {
    applySurfaceCompositing(material, compositing);
  }

  const isCasting: boolean = variant.pass === ERendererPass.DEFERRED && !variant.isImpostor;
  const isCutOut: boolean = variant.draw === ERendererDraw.CUT_OUT;
  // An impostor's texels are its atlas's, which the plain cut does not read.
  const isPickCut: boolean = isCutOut && !variant.isImpostor;

  material.pick = createPickMaterial({
    programs,
    shader: programs.getPick(isPickCut),
    slots: isPickCut ? slots.targets : null,
    source: material,
    uniforms,
    values: isPickCut ? values : null,
  });

  const shadow: Nullable<MeshBasicNodeMaterial> = !isCasting
    ? null
    : isCutOut
      ? createShadowMaterial(programs, uniforms, slots.targets, values)
      : opaqueShadow;

  return {
    dispose: () => {
      slots.release();
      material.pick?.dispose();
      material.dispose();

      // The opaque one is every opaque surface's, and goes with whatever made it.
      if (isCutOut) {
        shadow?.dispose();
      }
    },
    ...toOwnSurfaceDrawing(material, shadow, slots.keys),
    isImpostor: variant.isImpostor,
    keys: slots.keys,
    pass: variant.pass,
    shadowKeys: isCasting && isCutOut ? slots.keysOf(ESurfaceSlot.BASE) : [],
  };
}

/**
 * @param material - A surface's own material.
 * @param shadow - What casts it, or null for a surface that casts none.
 * @param keys - The texture keys both sample.
 * @returns What a surface drawing by them says of the static batches: that it draws them by them, reading no row,
 *   and draws its plain parts so too.
 */
export function toOwnSurfaceDrawing(
  material: MeshBasicNodeMaterial,
  shadow: Nullable<MeshBasicNodeMaterial>,
  keys: ReadonlyArray<string>
): Pick<ISurfaceMaterial, "batched" | "material" | "plain" | "row" | "shadow"> {
  return { batched: null, material, plain: { keys, material, shadow }, row: SURFACE_NO_ROW, shadow };
}

/**
 * @param variant - The variant of every surface it draws.
 * @param keys - The texture keys of the slots it samples of its own, which every surface it draws shares.
 * @param arrays - The arrays its array slots sample.
 * @param textures - Where its own slots are bound from.
 * @param uniforms - What the frame's shaders read.
 * @param programs - The shaders its variant shares.
 * @returns What draws every surface of the variant sharing its own textures in one static batch, each surface's
 *   numbers and array layers read from the surface table.
 */
export function createSurfaceBatchMaterial(
  variant: ISurfaceVariant,
  keys: IRendererSurface["textures"],
  arrays: TSurfaceArrayTargets,
  textures: RendererTextures,
  uniforms: RendererUniforms,
  programs: SurfacePrograms
): ISurfaceBatchMaterial {
  const arrayed: Array<ESurfaceSlot> = Object.keys(arrays) as Array<ESurfaceSlot>;
  const shader: ISurfaceShader = programs.getTabled(variant, arrayed);
  const slots: SurfaceSlots = new SurfaceSlots(
    textures,
    keys,
    toSampledSlots(variant).filter((slot: ESurfaceSlot) => !arrayed.includes(slot))
  );
  const material: SurfaceNodeMaterial = createSharedMaterial(programs, uniforms, slots.targets, null);

  material.surfaceArrays = arrays;
  applySurfaceShader(material, shader);

  const isCutOut: boolean = variant.draw === ERendererDraw.CUT_OUT && !variant.isImpostor;
  let shadow: Nullable<SurfaceNodeMaterial> = null;

  material.pick = createPickMaterial({
    programs,
    shader: isCutOut ? programs.getTabledPick(arrayed) : programs.getPick(false),
    slots: isCutOut ? slots.targets : null,
    source: material,
    uniforms,
    values: null,
  });
  material.pick.surfaceArrays = arrays;

  if (isCutOut) {
    shadow = createSharedMaterial(programs, uniforms, slots.targets, null);
    shadow.surfaceArrays = arrays;
    shadow.fragmentNode = programs.getTabledShadow(arrayed).fragmentNode ?? null;
    shadow.colorWrite = false;
    // Both faces, as every caster's: a card seen from the sun's side is its back as often as its front.
    shadow.side = DoubleSide;
  }

  return {
    dispose: () => {
      slots.release();
      material.pick?.dispose();
      material.dispose();
      shadow?.dispose();
    },
    keys: slots.keys,
    material,
    shadow,
    shadowKeys: isCutOut ? slots.keysOf(ESurfaceSlot.BASE) : [],
  };
}

/**
 * @param programs - The shaders the surfaces share.
 * @param uniforms - What the frame's shaders read.
 * @returns The shadow material every opaque surface shares: its depth alone, whatever it is dressed with, so every
 *   opaque caster of an arena is one batch.
 */
export function createOpaqueShadowMaterial(
  programs: SurfacePrograms,
  uniforms: RendererUniforms
): MeshBasicNodeMaterial {
  return createShadowMaterial(programs, uniforms, null, null);
}

/**
 * @param programs - The shaders the surfaces share.
 * @param uniforms - What the frame's shaders read.
 * @param slots - A cut-out surface's slots, shared with its G-buffer material, or null for the opaque surfaces'.
 * @param values - Its numbers, likewise.
 * @returns Its depth-only material, standing its geometry where the G-buffer's does.
 */
function createShadowMaterial(
  programs: SurfacePrograms,
  uniforms: RendererUniforms,
  slots: Nullable<TSurfaceSlotTargets>,
  values: Nullable<ISurfaceValues>
): MeshBasicNodeMaterial {
  const material: SurfaceNodeMaterial = createSharedMaterial(programs, uniforms, slots, values);

  material.fragmentNode = programs.getShadow(slots !== null).fragmentNode ?? null;
  material.colorWrite = false;
  // Both faces: a card seen from the sun's side is its back as often as its front, and a wall casts either way.
  material.side = DoubleSide;

  return material;
}

/** What a surface's pick twin is made from. */
interface IPickMaterialInput {
  programs: SurfacePrograms;
  uniforms: RendererUniforms;
  /** The material it is the twin of, whose vertices it stands where they stand and whose faces it draws. */
  source: SurfaceNodeMaterial;
  /** What draws it into a pick. */
  shader: ISurfaceShader;
  /** A cut-out surface's slots, shared with its G-buffer material, or null for any other. */
  slots: Nullable<TSurfaceSlotTargets>;
  /** Its numbers, likewise. */
  values: Nullable<ISurfaceValues>;
}

/**
 * @param input - What it is made from.
 * @returns The material a pick draws a surface with in place of its own: standing its geometry as the source does, and
 *   writing which draw it is rather than how it looks.
 */
function createPickMaterial(input: IPickMaterialInput): SurfaceNodeMaterial {
  const { programs, uniforms, source, shader, slots, values } = input;
  const material: SurfaceNodeMaterial = createSharedMaterial(programs, uniforms, slots, values);

  material.fragmentNode = shader.fragmentNode ?? null;
  material.positionViewNode = source.positionViewNode;
  material.side = source.side;

  return material;
}

/** Has a material draw with a shader's nodes. */
function applySurfaceShader(material: SurfaceNodeMaterial, shader: ISurfaceShader): void {
  material.fragmentNode = shader.fragmentNode ?? null;
  material.colorNode = shader.colorNode ?? null;
  material.alphaTestNode = shader.alphaTestNode ?? null;
  material.positionViewNode = shader.positionViewNode ?? null;
}

/** A material drawing with its variant's shared nodes, carrying what they read of it. */
function createSharedMaterial(
  programs: SurfacePrograms,
  uniforms: RendererUniforms,
  slots: Nullable<TSurfaceSlotTargets>,
  values: Nullable<ISurfaceValues>
): SurfaceNodeMaterial {
  const material: SurfaceNodeMaterial = new SurfaceNodeMaterial(uniforms.staticDraws, uniforms.treeWind);

  // Every surface stands its geometry in each place instanced attributes name, and in its own place where none do.
  material.positionNode = programs.position;
  material.surfaceSlots = slots;
  material.surfaceValues = values;

  return material;
}
