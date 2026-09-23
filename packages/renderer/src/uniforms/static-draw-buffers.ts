import { IndirectStorageBufferAttribute, StorageBufferAttribute } from "three/webgpu";

/** Static draws the buffers hold, which is more sections than any level measured puts in its resident sectors. */
export const STATIC_DRAW_CAPACITY: number = 65536;

/** Unsigned integers one indirect draw takes: index count, instance count, first index, base vertex, first instance. */
export const STATIC_DRAW_ARGUMENTS: number = 5;

/**
 * What every static draw reads and is drawn by, one slot a draw: the arguments a compute pass culls, the sphere it
 * culls by, and the matrix the draw's shader places it with. One set of buffers for every static draw, so no draw has
 * a uniform of its own to refresh. A draw's first instance is its slot, which its shader reads back through its
 * arena's slot attribute (`EVertexAttribute.STATIC_SLOT`), every element of which holds its own number.
 */
export class StaticDrawBuffers {
  /** Indirect arguments, the instance count of each written by the cull on the GPU. */
  public readonly args: IndirectStorageBufferAttribute = new IndirectStorageBufferAttribute(
    new Uint32Array(STATIC_DRAW_CAPACITY * STATIC_DRAW_ARGUMENTS),
    STATIC_DRAW_ARGUMENTS
  );
  /** Each draw's sphere in renderer space, a negative radius for a slot drawing nothing. */
  public readonly spheres: StorageBufferAttribute = new StorageBufferAttribute(
    new Float32Array(STATIC_DRAW_CAPACITY * 4).fill(-1),
    4
  );
  /** Each draw's matrix, four columns. */
  public readonly models: StorageBufferAttribute = new StorageBufferAttribute(
    new Float32Array(STATIC_DRAW_CAPACITY * 16),
    4
  );
  /** What the last cull kept: draws, then indices, for the frame report. */
  public readonly counts: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(2), 1);
}
