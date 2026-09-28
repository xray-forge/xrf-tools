import { IndirectStorageBufferAttribute, StorageBufferAttribute } from "three/webgpu";

import { IRendererGrass, IRendererGrassModel } from "#/contract/scene/renderer-grass";
import { createGrassDither, GRASS_ITEM_VECTORS } from "#/scene/grass/grass-planting.tsl";
import { STATIC_DRAW_ARGUMENTS } from "#/uniforms/static-draw-buffers";

/** What the grass is planted from, and what each model's count and draw are kept in: the level's, made once. */
export interface IGrassLevelBuffers {
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
  /** Metres the largest tuft reaches past the ground it stands on, at a height of one: what a slot's box grows by. */
  tuftReach: number;
  /** Items planted a model, then the whole frame's in the last. */
  counts: StorageBufferAttribute;
  /** Where each model's range goes on filling, as the items are sorted into it. */
  cursors: StorageBufferAttribute;
  /** Where each model's range starts among the sorted items, read by its draw rather than drawn as a first instance. */
  starts: StorageBufferAttribute;
  /** One draw a model. */
  args: IndirectStorageBufferAttribute;
}

/** The frame's planted items, as many as the planting's settings ask room for. */
export interface IGrassItemBuffers {
  /** Items the lists hold. */
  capacity: number;
  /** The frame's items as they were planted, and which model each is. */
  items: StorageBufferAttribute;
  itemModels: StorageBufferAttribute;
  /** The same items sorted by model, which the draws read. */
  sorted: StorageBufferAttribute;
}

/** Everything the planting reads and writes. */
export type TGrassBuffers = IGrassLevelBuffers & IGrassItemBuffers;

/**
 * @param grass - The level's grass.
 * @returns What it is planted from.
 */
export function createGrassLevelBuffers(grass: IRendererGrass): IGrassLevelBuffers {
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
    starts: new StorageBufferAttribute(new Uint32Array(Math.max(modelCount, 1)), 1),
    dither: new StorageBufferAttribute(createGrassDither(), 1),
    grid: toStorage(grass.grid, 1),
    gridLength: Math.max(grass.grid.length, 1),
    indexCounts: grass.models.map((model: IRendererGrassModel) => model.indices.length),
    modelCount,
    models: new StorageBufferAttribute(models, 4),
    slotWords: Math.max(grass.slots.length, 1),
    slots: toStorage(grass.slots, 1),
    triangleFloats: Math.max(grass.triangles.length, 1),
    triangles: toStorage(grass.triangles, 1),
    tuftReach: grass.models.reduce(
      (reach: number, model: IRendererGrassModel) => Math.max(reach, model.maxScale * (model.radius + model.height)),
      0
    ),
  };
}

/**
 * @param capacity - Items the lists hold.
 * @returns The lists.
 */
export function createGrassItemBuffers(capacity: number): IGrassItemBuffers {
  return {
    capacity,
    itemModels: new StorageBufferAttribute(new Uint32Array(capacity), 1),
    items: new StorageBufferAttribute(new Float32Array(capacity * GRASS_ITEM_VECTORS * 4), 4),
    sorted: new StorageBufferAttribute(new Float32Array(capacity * GRASS_ITEM_VECTORS * 4), 4),
  };
}

/**
 * Items the lists are made to hold for a need: the next power of two, so a setting dragged up rebuilds them a handful of
 * times rather than at every step, and one brought back down rebuilds nothing. Never past what one storage buffer may
 * hold: a need past it plants what fits, the planting dropping the rest.
 *
 * @param needed - Items the settings plant at most.
 * @param storageLimit - Bytes one storage buffer may hold and be bound whole.
 * @returns The capacity to make the lists with.
 */
export function toGrassItemCapacity(needed: number, storageLimit: number): number {
  const rounded: number = 2 ** Math.ceil(Math.log2(Math.max(needed, 1)));
  const most: number = Math.floor(storageLimit / (GRASS_ITEM_VECTORS * 16));

  return Math.max(1, Math.min(rounded, most));
}

/**
 * @param buffers - The level's buffers.
 * @returns Every storage buffer among them, for letting them go.
 */
export function listGrassLevelStorage(buffers: IGrassLevelBuffers): Array<StorageBufferAttribute> {
  const { counts, cursors, starts, grid, slots, bins, triangles, dither, models, args } = buffers;

  return [counts, cursors, starts, grid, slots, bins, triangles, dither, models, args];
}

/**
 * @param buffers - The item lists.
 * @returns Every storage buffer among them, for letting them go.
 */
export function listGrassItemStorage(buffers: IGrassItemBuffers): Array<StorageBufferAttribute> {
  return [buffers.items, buffers.itemModels, buffers.sorted];
}

/** A storage attribute over an array, one element long where the array is empty: a binding cannot be. */
function toStorage(array: Uint32Array | Float32Array, itemSize: number): StorageBufferAttribute {
  if (array.length) {
    return new StorageBufferAttribute(array, itemSize);
  }

  return new StorageBufferAttribute(array instanceof Uint32Array ? new Uint32Array(1) : new Float32Array(1), itemSize);
}
