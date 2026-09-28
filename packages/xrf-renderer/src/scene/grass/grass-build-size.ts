import { toGrassCachePerCell } from "#/scene/grass/grass-cache-buffers";
import { toGrassItemCapacity } from "#/scene/grass/grass-item-buffers";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

/** How large a build of the grass's buffers is: its item lists, and its ring. */
export interface IGrassBuildSize {
  /** Items the lists hold. */
  capacity: number;
  /** Slots the ring holds. */
  cells: number;
  /** Tufts a cell holds at most. */
  perCell: number;
  /** Rings of cells around the camera's. */
  bands: number;
}

/**
 * @param uniforms - How far and how densely the planting reaches.
 * @param storageLimit - Bytes one storage buffer may hold and be bound whole.
 * @returns What the settings want a build to hold.
 */
export function toGrassBuildSize(uniforms: GrassUniforms, storageLimit: number): IGrassBuildSize {
  const cells: number = uniforms.slotCount;

  return {
    bands: uniforms.reach.value + 1,
    capacity: toGrassItemCapacity(cells * uniforms.candidateCount, storageLimit),
    cells,
    perCell: toGrassCachePerCell(cells, uniforms.candidateCount, storageLimit),
  };
}

/**
 * A build holds a smaller want whole: a ring of fewer cells takes the first of them, and bands and lists past the want
 * go unused. Only a want past it builds again.
 *
 * @param held - What a build holds.
 * @param wanted - What the settings want.
 * @returns Whether the build is too small for it.
 */
export function isGrassBuildOutgrown(held: IGrassBuildSize, wanted: IGrassBuildSize): boolean {
  return (
    wanted.capacity > held.capacity ||
    wanted.cells > held.cells ||
    wanted.perCell > held.perCell ||
    wanted.bands > held.bands
  );
}
