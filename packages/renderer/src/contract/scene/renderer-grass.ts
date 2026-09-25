import { IRendererSurface } from "#/contract/scene/renderer-surface";

/** `u32` words a planted slot takes: its stored sixteen bytes, its triangle bin's start and length, its world slot. */
export const RENDERER_GRASS_SLOT_WORDS: number = 8;

/** Floats a collision triangle takes: its three corners. */
export const RENDERER_GRASS_TRIANGLE_FLOATS: number = 9;

/**
 * One detail model of a level's library, which the grass plants.
 */
export interface IRendererGrassModel {
  /** Three floats a vertex, in renderer space. */
  positions: Float32Array;
  /** Two floats a vertex. */
  uvs: Float32Array;
  indices: Uint16Array;
  /** What it is dressed with: its base texture, cut out at its reference. */
  surface: IRendererSurface;
  /** Whether the wind moves it. */
  isWaving: boolean;
  /** The scale range it is planted at, as the library states it. */
  minScale: number;
  maxScale: number;
  /** Its bounding box's height, which a vertex's share of the sway is measured against. */
  height: number;
  /** The radius of the sphere around its bounding box. */
  radius: number;
}

/**
 * A level's grass as the engine plants it (`CDetailManager`): a grid of two metre slots, each naming up to four
 * models and how densely each grows at each corner, planted onto the collision triangles binned under it. The slots and
 * triangles are in the engine's own space, since the renderer plants them with the engine's own arithmetic.
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
  /** `RENDERER_GRASS_TRIANGLE_FLOATS` a triangle, in the engine's space and winding. */
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
  ].map((array) => array.buffer);
}
