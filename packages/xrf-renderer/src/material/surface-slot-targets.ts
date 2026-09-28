import { ESurfaceSlot } from "#/material/surface-slot";
import { ITextureTarget } from "#/texture/texture-target";

/** What a material's slots draw, each pointed at its key's texture as that uploads. */
export type TSurfaceSlotTargets = Readonly<Record<ESurfaceSlot, ITextureTarget>>;
