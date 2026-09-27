import { Nullable } from "@xrf/types";
import { DoubleSide, MeshBasicNodeMaterial } from "three/webgpu";

import { ERendererDraw, ERendererPass, IRendererSurface } from "#/contract/scene/renderer-surface";
import { applySurfaceCompositing, ISurfaceCompositing, toSurfaceCompositing } from "#/material/surface-compositing";
import { SurfaceNodeMaterial } from "#/material/surface-node-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { ISurfaceShader } from "#/material/surface-shader";
import { ESurfaceSlot, TSurfaceSlotTargets } from "#/material/surface-slot";
import { SurfaceSlots } from "#/material/surface-slots";
import { ISurfaceValues, toSurfaceValues } from "#/material/surface-values";
import { ISurfaceVariant, toSampledSlots, toSurfaceVariant } from "#/material/surface-variant";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * A surface as the frame draws it.
 */
export interface ISurfaceMaterial {
  material: MeshBasicNodeMaterial;
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

  material.fragmentNode = shader.fragmentNode ?? null;
  material.colorNode = shader.colorNode ?? null;
  material.alphaTestNode = shader.alphaTestNode ?? null;
  material.positionViewNode = shader.positionViewNode ?? null;

  if (variant.isImpostor) {
    // Its quad turns to face the camera, from whichever side it is seen.
    material.side = DoubleSide;
  }

  if (compositing) {
    applySurfaceCompositing(material, compositing);
  }

  const isCasting: boolean = variant.pass === ERendererPass.DEFERRED && !variant.isImpostor;
  const isCutOut: boolean = variant.draw === ERendererDraw.CUT_OUT;
  const shadow: Nullable<MeshBasicNodeMaterial> = !isCasting
    ? null
    : isCutOut
      ? createShadowMaterial(programs, uniforms, slots.targets, values)
      : opaqueShadow;

  return {
    dispose: () => {
      slots.release();
      material.dispose();

      // The opaque one is every opaque surface's, and goes with whatever made it.
      if (isCutOut) {
        shadow?.dispose();
      }
    },
    isImpostor: variant.isImpostor,
    keys: slots.keys,
    material,
    pass: variant.pass,
    shadow,
    shadowKeys: isCasting && isCutOut ? slots.keysOf(ESurfaceSlot.BASE) : [],
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
