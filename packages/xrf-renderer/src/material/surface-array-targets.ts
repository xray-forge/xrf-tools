import { ESurfaceSlot } from "#/material/surface-slot";
import { ITextureTarget } from "#/texture/texture-target";

/** What a shared material's array slots sample: each slot's array as it is now, for the slots it samples so. */
export type TSurfaceArrayTargets = Readonly<Partial<Record<ESurfaceSlot, ITextureTarget>>>;
