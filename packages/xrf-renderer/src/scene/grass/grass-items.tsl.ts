import { storage } from "three/tsl";
import { StorageBufferNode } from "three/webgpu";

import { GRASS_ITEM_VECTORS, IGrassItemBuffers } from "#/scene/grass/grass-item-buffers";
import { IGrassLevelBuffers } from "#/scene/grass/grass-level-buffers";

/**
 * @param level - The level's buffers.
 * @returns Items planted a model, then the whole frame's in the last, atomically.
 */
export function toGrassCounts(level: IGrassLevelBuffers): StorageBufferNode<"uint"> {
  return storage(level.counts, "uint", level.modelCount + 1).toAtomic() as never;
}

/**
 * @param level - The level's buffers.
 * @returns Where each model's range goes on filling, atomically.
 */
export function toGrassCursors(level: IGrassLevelBuffers): StorageBufferNode<"uint"> {
  return storage(level.cursors, "uint", Math.max(level.modelCount, 1)).toAtomic() as never;
}

/**
 * @param level - The level's buffers.
 * @returns Where each model's tufts start among the sorted items, as its draw reads them.
 */
export function toGrassStarts(level: IGrassLevelBuffers): StorageBufferNode<"uint"> {
  return storage(level.starts, "uint", Math.max(level.modelCount, 1)).toReadOnly() as never;
}

/**
 * @param items - The item lists.
 * @returns The frame's items in the order they were planted.
 */
export function toGrassPlanted(items: IGrassItemBuffers): StorageBufferNode<"vec4"> {
  return storage(items.planted, "vec4", items.capacity * GRASS_ITEM_VECTORS) as never;
}

/**
 * @param items - The item lists.
 * @returns The model each planted item is.
 */
export function toGrassPlantedModels(items: IGrassItemBuffers): StorageBufferNode<"uint"> {
  return storage(items.models, "uint", items.capacity) as never;
}

/**
 * @param items - The item lists.
 * @returns The items sorted by model.
 */
export function toGrassSorted(items: IGrassItemBuffers): StorageBufferNode<"vec4"> {
  return storage(items.sorted, "vec4", items.capacity * GRASS_ITEM_VECTORS) as never;
}
