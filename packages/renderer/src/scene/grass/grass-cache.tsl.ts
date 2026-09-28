import { storage } from "three/tsl";
import { StorageBufferNode } from "three/webgpu";

import {
  GRASS_CACHE_KEY_WORDS,
  GRASS_CACHE_VECTORS,
  GRASS_SCHEDULE_WORDS,
  IGrassCacheBuffers,
} from "#/scene/grass/grass-cache-buffers";

/**
 * @param cache - The ring.
 * @returns Its keys, `GRASS_CACHE_KEY_WORDS` a cell.
 */
export function toGrassCacheKeys(cache: IGrassCacheBuffers): StorageBufferNode<"uint"> {
  return storage(cache.keys, "uint", cache.cells * GRASS_CACHE_KEY_WORDS) as never;
}

/**
 * @param cache - The ring.
 * @returns A vector a cell: its ground's middle height and half its height, its hemisphere and its sun.
 */
export function toGrassCacheShapes(cache: IGrassCacheBuffers): StorageBufferNode<"vec4"> {
  return storage(cache.shapes, "vec4", cache.cells) as never;
}

/**
 * @param cache - The ring.
 * @returns Its tufts, `GRASS_CACHE_VECTORS` each, `perCell` a cell.
 */
export function toGrassCacheItems(cache: IGrassCacheBuffers): StorageBufferNode<"vec4"> {
  return storage(cache.items, "vec4", cache.cells * cache.perCell * GRASS_CACHE_VECTORS) as never;
}

/**
 * @param cache - The ring.
 * @returns The stale cells each band counts, atomically.
 */
export function toGrassBandCounts(cache: IGrassCacheBuffers): StorageBufferNode<"uint"> {
  return storage(cache.bandCounts, "uint", cache.bands).toAtomic() as never;
}

/**
 * @param cache - The ring.
 * @returns The schedule, `GRASS_SCHEDULE_WORDS`, atomically.
 */
export function toGrassSchedule(cache: IGrassCacheBuffers): StorageBufferNode<"uint"> {
  return storage(cache.schedule, "uint", GRASS_SCHEDULE_WORDS).toAtomic() as never;
}
