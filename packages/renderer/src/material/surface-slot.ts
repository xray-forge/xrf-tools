import { Nullable } from "@xrf/types";
import { Texture } from "three/webgpu";

import {
  getClearTexture,
  getFlatBumpCompanionTexture,
  getFlatBumpTexture,
  getFlatNormalTexture,
  getNeutralDetailTexture,
  getWhiteTexture,
} from "#/texture/placeholder-textures";
import { ITextureTarget } from "#/texture/texture-target";

/**
 * The textures a surface's shader samples, named as the surface's own textures are.
 */
export enum ESurfaceSlot {
  BASE = "base",
  DETAIL = "detail",
  BUMP = "bump",
  BUMP_COMPANION = "bumpCompanion",
  HEMI = "hemi",
  NORMAL = "normal",
  FOAM = "foam",
  DISTORTION = "distortion",
}

/** Every slot, in the order a material binds them. */
export const SURFACE_SLOTS: ReadonlyArray<ESurfaceSlot> = Object.values(ESurfaceSlot);

/**
 * @param slot - The slot.
 * @returns What it samples while it holds nothing: what leaves the surface as it would be without it.
 */
export function getSurfaceSlotPlaceholder(slot: ESurfaceSlot): Texture {
  switch (slot) {
    case ESurfaceSlot.DETAIL:
    case ESurfaceSlot.DISTORTION:
      return getNeutralDetailTexture();

    case ESurfaceSlot.NORMAL:
      return getFlatNormalTexture();

    case ESurfaceSlot.FOAM:
      return getClearTexture();

    case ESurfaceSlot.BUMP:
      return getFlatBumpTexture();

    case ESurfaceSlot.BUMP_COMPANION:
      return getFlatBumpCompanionTexture();

    case ESurfaceSlot.BASE:
    case ESurfaceSlot.HEMI:
      return getWhiteTexture();
  }
}

/** What a material's slots draw, each pointed at its key's texture as that uploads. */
export type TSurfaceSlotTargets = Readonly<Record<ESurfaceSlot, ITextureTarget>>;

/** What a shared material's array slots sample: each slot's array as it is now, for the slots it samples so. */
export type TSurfaceArrayTargets = Readonly<Partial<Record<ESurfaceSlot, ITextureTarget>>>;

/** A material whose slots a shared shader samples. */
export interface ISurfaceSlotted {
  surfaceSlots: Nullable<TSurfaceSlotTargets>;
  /** The arrays its array slots sample, or null for a material sampling every slot from a texture of its own. */
  surfaceArrays: Nullable<TSurfaceArrayTargets>;
}
