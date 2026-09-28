import { Vector3 } from "three/webgpu";

import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { DEFAULT_MATERIAL, MATERIAL_SLICES } from "#/material/surface-texel.tsl";
import { DEFAULT_ALPHA_REFERENCE } from "#/shader/alpha-cut.tsl";

/**
 * The numbers one surface states, which its material carries and its variant's shader reads for each object drawn.
 */
export interface ISurfaceValues {
  /** How many times the base and every other texture repeat. */
  tiling: number;
  /** Detail repeats per base repeat. */
  detailScale: number;
  /** Where a cut-out or alpha-tested texel is cut, `def_aref` where the surface names none. */
  alphaReference: number;
  /** The lighting model's slice of the material lookup. */
  slice: number;
  /** What the base is multiplied by. */
  color: Vector3;
}

/**
 * @param surface - What the consumer put.
 * @returns Its numbers, each at the default the engine takes where it gives none.
 */
export function toSurfaceValues(surface: IRendererSurface): ISurfaceValues {
  return {
    alphaReference: surface.alphaReference ?? DEFAULT_ALPHA_REFERENCE,
    color: surface.color ? new Vector3(...surface.color) : new Vector3(1, 1, 1),
    detailScale: surface.detailScale ?? 1,
    slice: ((surface.material ?? DEFAULT_MATERIAL) + 0.5) / MATERIAL_SLICES,
    tiling: surface.tiling ?? 1,
  };
}
