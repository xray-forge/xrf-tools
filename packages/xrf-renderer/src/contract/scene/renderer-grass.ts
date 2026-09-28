import { IRendererGrassModel } from "#/contract/scene/renderer-grass-model";

/** `u32` words a planted slot takes: its stored sixteen bytes, then its triangle bin's start and length. */
export const RENDERER_GRASS_SLOT_WORDS: number = 6;

/** Floats a collision triangle takes: its three corners. */
export const RENDERER_GRASS_TRIANGLE_FLOATS: number = 9;

/**
 * A level's grass as the engine plants it (`CDetailManager`): a grid of two metre slots, each naming up to four
 * models and how densely each grows at each corner, planted onto the collision triangles binned under it. The slots are
 * the engine's own, since the renderer plants them with the engine's own arithmetic; the triangles are in renderer
 * space, as every position the renderer is handed is.
 */
export interface IRendererGrass {
  /** The grid's size in slots, and how far its first cell stands from world slot zero. */
  sizeX: number;
  sizeZ: number;
  offsetX: number;
  offsetZ: number;
  /** One word a cell, `z * sizeX + x`: its planted slot's record plus one, zero for nothing to plant. */
  grid: Uint32Array;
  /** `RENDERER_GRASS_SLOT_WORDS` a planted slot. */
  slots: Uint32Array;
  /** One word an entry: a triangle, by its index. */
  bins: Uint32Array;
  /** `RENDERER_GRASS_TRIANGLE_FLOATS` a triangle, in renderer space, wound for it. */
  triangles: Float32Array;
  models: ReadonlyArray<IRendererGrassModel>;
}

/**
 * What of the grass moves between threads: its arrays, never copied.
 *
 * @param grass - The grass about to be posted.
 * @returns Its buffers.
 */
export function listRendererGrassTransfers(grass: IRendererGrass): Array<Transferable> {
  return [
    grass.grid,
    grass.slots,
    grass.bins,
    grass.triangles,
    ...grass.models.flatMap((model: IRendererGrassModel) => [model.positions, model.uvs, model.indices]),
  ].map((array: ArrayBufferView) => array.buffer);
}
