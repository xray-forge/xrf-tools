import { StorageBufferAttribute } from "three/webgpu";

/** Vectors of four floats an item takes: its place and turn, then its scale, light and wave. */
export const GRASS_ITEM_VECTORS: number = 2;

/** The frame's planted items, as many as the planting's settings ask room for. */
export interface IGrassItemBuffers {
  /** Items the lists hold. */
  capacity: number;
  /** The frame's items in the order they were planted. */
  planted: StorageBufferAttribute;
  /** The model each planted item is. */
  models: StorageBufferAttribute;
  /** The same items sorted by model, which the draws read. */
  sorted: StorageBufferAttribute;
}

/**
 * @param capacity - Items the lists hold.
 * @returns The lists.
 */
export function createGrassItemBuffers(capacity: number): IGrassItemBuffers {
  return {
    capacity,
    models: new StorageBufferAttribute(new Uint32Array(capacity), 1),
    planted: new StorageBufferAttribute(new Float32Array(capacity * GRASS_ITEM_VECTORS * 4), 4),
    sorted: new StorageBufferAttribute(new Float32Array(capacity * GRASS_ITEM_VECTORS * 4), 4),
  };
}

/**
 * Items the lists are made to hold for a need: the next power of two, so a setting dragged up or down rebuilds them a
 * handful of times rather than at every step. Never past what one storage buffer may hold: a need past it plants what
 * fits, the planting dropping the rest.
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
 * @param buffers - The item lists.
 * @returns Every storage buffer among them, for letting them go.
 */
export function listGrassItemStorage(buffers: IGrassItemBuffers): Array<StorageBufferAttribute> {
  return [buffers.planted, buffers.models, buffers.sorted];
}
