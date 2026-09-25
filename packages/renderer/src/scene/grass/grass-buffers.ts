import { IndirectStorageBufferAttribute, StorageBufferAttribute } from "three/webgpu";

import { IRendererGrass, IRendererGrassModel } from "#/contract/scene/renderer-grass";
import { createGrassDither, GRASS_ITEM_VECTORS } from "#/scene/grass/grass-planting.tsl";
import { STATIC_DRAW_ARGUMENTS } from "#/uniforms/static-draw-buffers";

/** What the grass is planted from and into, on the GPU. */
export interface IGrassBuffers {
  modelCount: number;
  /** Each model's index count, which its draw is written with. */
  indexCounts: ReadonlyArray<number>;
  grid: StorageBufferAttribute;
  gridLength: number;
  slots: StorageBufferAttribute;
  slotWords: number;
  bins: StorageBufferAttribute;
  binLength: number;
  triangles: StorageBufferAttribute;
  triangleFloats: number;
  dither: StorageBufferAttribute;
  /** Two vectors a model: its least and most scale, radius and height, then whether it waves. */
  models: StorageBufferAttribute;
  /** Items planted a model, then the whole frame's in the last. */
  counts: StorageBufferAttribute;
  /** Where each model's range goes on filling, as the items are sorted into it. */
  cursors: StorageBufferAttribute;
  /** The frame's items as they were planted, and which model each is. */
  items: StorageBufferAttribute;
  itemModels: StorageBufferAttribute;
  /** The same items sorted by model, which the draws read. */
  sorted: StorageBufferAttribute;
  /** One draw a model. */
  args: IndirectStorageBufferAttribute;
}

/**
 * @param grass - The level's grass.
 * @param capacity - Items the lists hold.
 * @returns Its buffers.
 */
export function createGrassBuffers(grass: IRendererGrass, capacity: number): IGrassBuffers {
  const modelCount: number = grass.models.length;
  const models: Float32Array = new Float32Array(Math.max(modelCount, 1) * 8);

  grass.models.forEach((model: IRendererGrassModel, index: number) => {
    models.set(
      [model.minScale, model.maxScale, model.radius, model.height, model.isWaving ? 1 : 0, 0, 0, 0],
      index * 8
    );
  });

  return {
    args: new IndirectStorageBufferAttribute(new Uint32Array(Math.max(modelCount, 1) * STATIC_DRAW_ARGUMENTS), 1),
    binLength: Math.max(grass.bins.length, 1),
    bins: toStorage(grass.bins, 1),
    counts: new StorageBufferAttribute(new Uint32Array(modelCount + 1), 1),
    cursors: new StorageBufferAttribute(new Uint32Array(Math.max(modelCount, 1)), 1),
    dither: new StorageBufferAttribute(createGrassDither(), 1),
    grid: toStorage(grass.grid, 1),
    gridLength: Math.max(grass.grid.length, 1),
    indexCounts: grass.models.map((model: IRendererGrassModel) => model.indices.length),
    itemModels: new StorageBufferAttribute(new Uint32Array(capacity), 1),
    items: new StorageBufferAttribute(new Float32Array(capacity * GRASS_ITEM_VECTORS * 4), 4),
    modelCount,
    models: new StorageBufferAttribute(models, 4),
    slotWords: Math.max(grass.slots.length, 1),
    slots: toStorage(grass.slots, 1),
    sorted: new StorageBufferAttribute(new Float32Array(capacity * GRASS_ITEM_VECTORS * 4), 4),
    triangleFloats: Math.max(grass.triangles.length, 1),
    triangles: toStorage(grass.triangles, 1),
  };
}

/** A storage attribute over an array, one element long where the array is empty: a binding cannot be. */
function toStorage(array: Uint32Array | Float32Array, itemSize: number): StorageBufferAttribute {
  if (array.length) {
    return new StorageBufferAttribute(array, itemSize);
  }

  return new StorageBufferAttribute(array instanceof Uint32Array ? new Uint32Array(1) : new Float32Array(1), itemSize);
}
