import { StorageBufferAttribute } from "three/webgpu";

/** Vectors of four floats a cached tuft takes: its place and turn, then its size, model and wave. */
export const GRASS_CACHE_VECTORS: number = 2;

/** Words a cached slot's key takes: the world slot it holds on each axis, the generation it was planted under, its tufts. */
export const GRASS_CACHE_KEY_WORDS: number = 4;

/** Words the schedule keeps: the band a frame's planting stops in, the slots it takes there, and its tickets. */
export const GRASS_SCHEDULE_WORDS: number = 3;

/**
 * `CDetailManager`'s cache on the GPU: a ring of slots around the camera, a cell each, holding what each was planted
 * with, and what schedules the planting of the cells that fell stale.
 */
export interface IGrassCacheBuffers {
  /** Slots the ring holds: as many as the planting reaches, squared. */
  cells: number;
  /** Tufts a cell holds at most. */
  perCell: number;
  /** Rings of cells around the camera's, the nearest first: the planting's reach at most, and one. */
  bands: number;
  /** `GRASS_CACHE_KEY_WORDS` a cell. */
  keys: StorageBufferAttribute;
  /** A vector a cell: its ground's middle height and half its height, its hemisphere and its sun. */
  shapes: StorageBufferAttribute;
  /** `GRASS_CACHE_VECTORS` a cached tuft: its place and turn, then its size, model and wave. */
  items: StorageBufferAttribute;
  /** Stale cells a band holds, counted each frame. */
  bandCounts: StorageBufferAttribute;
  /** `GRASS_SCHEDULE_WORDS`. */
  schedule: StorageBufferAttribute;
}

/**
 * The ring made empty: every key holds generation nought, which no planting runs under, so every cell starts stale.
 *
 * @param cells - Slots the ring holds.
 * @param perCell - Tufts a cell holds at most.
 * @param bands - Rings of cells around the camera's.
 * @returns The cache.
 */
export function createGrassCacheBuffers(cells: number, perCell: number, bands: number): IGrassCacheBuffers {
  return {
    bandCounts: new StorageBufferAttribute(new Uint32Array(bands), 1),
    bands,
    cells,
    items: new StorageBufferAttribute(new Float32Array(cells * perCell * GRASS_CACHE_VECTORS * 4), 4),
    keys: new StorageBufferAttribute(new Uint32Array(cells * GRASS_CACHE_KEY_WORDS), 1),
    perCell,
    schedule: new StorageBufferAttribute(new Uint32Array(GRASS_SCHEDULE_WORDS), 1),
    shapes: new StorageBufferAttribute(new Float32Array(cells * 4), 4),
  };
}

/**
 * Tufts a cell is made to hold: every candidate the settings lay out, or fewer where the ring would outgrow one storage
 * buffer, the planting dropping the rest.
 *
 * @param cells - Slots the ring holds.
 * @param candidates - Candidates a slot lays out.
 * @param storageLimit - Bytes one storage buffer may hold and be bound whole.
 * @returns Tufts a cell holds.
 */
export function toGrassCachePerCell(cells: number, candidates: number, storageLimit: number): number {
  return Math.max(1, Math.min(candidates, Math.floor(storageLimit / (Math.max(cells, 1) * GRASS_CACHE_VECTORS * 16))));
}

/**
 * @param buffers - The cache.
 * @returns Every storage buffer it holds, for letting them go.
 */
export function listGrassCacheStorage(buffers: IGrassCacheBuffers): Array<StorageBufferAttribute> {
  return [buffers.keys, buffers.shapes, buffers.items, buffers.bandCounts, buffers.schedule];
}
