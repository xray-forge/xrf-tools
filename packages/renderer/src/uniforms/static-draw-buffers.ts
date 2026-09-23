import { storage } from "three/tsl";
import { IndirectStorageBufferAttribute, StorageBufferAttribute } from "three/webgpu";

import { OcclusionUniforms } from "#/uniforms/occlusion-uniforms";

/** Static draws the buffers hold, which is more sections than any level measured puts in its resident sectors. */
export const STATIC_DRAW_CAPACITY: number = 65536;

/** Unsigned integers one indirect draw takes: index count, instance count, first index, base vertex, first instance. */
export const STATIC_DRAW_ARGUMENTS: number = 5;

/** Places instanced static draws stand in, which is several times the trees any level measured puts in view. */
export const STATIC_PLACE_CAPACITY: number = 1 << 16;

/** Columns one place takes: its matrix's four, then its hemisphere scale and offset. */
export const STATIC_PLACE_COLUMNS: number = 5;

/** Rows the instance cull tests, a place of one instanced draw each; as many entries in each list it writes. */
export const STATIC_ROW_CAPACITY: number = 1 << 17;

/** Depths the pyramid holds over all its levels: enough for a drawing of 4096 by 4096. */
export const STATIC_PYRAMID_CAPACITY: number = 1 << 20;

/** What the first cull decided for a slot or row: out of view, drawn, or left for the second cull as occluded. */
export enum EStaticCullState {
  OUTSIDE = 0,
  DRAWN = 1,
  OCCLUDED = 2,
}

/**
 * What every static draw reads and is drawn by, one slot a draw: the arguments a compute pass culls, the sphere it
 * culls by, and the matrix the draw's shader places it with. One set of buffers for every static draw, so no draw has
 * a uniform of its own to refresh. A single draw's first instance is its slot, which its shader reads back through
 * its arena's slot attribute (`EVertexAttribute.STATIC_SLOT`), every element of which holds its own number. An
 * instanced draw's first instance is where its list of kept places starts: a place per instance, whose matrix and
 * hemisphere terms its shader reads.
 */
export class StaticDrawBuffers {
  /** Indirect arguments of the first draw, the instance count of each written by the first cull on the GPU. */
  public readonly args: IndirectStorageBufferAttribute = new IndirectStorageBufferAttribute(
    new Uint32Array(STATIC_DRAW_CAPACITY * STATIC_DRAW_ARGUMENTS),
    STATIC_DRAW_ARGUMENTS
  );
  /**
   * The same arguments for the second draw, of what the first cull found occluded by the last frame's depth and the
   * second finds seen by this frame's: the instance counts are the second cull's, an instanced draw lists its places
   * in the second half of the list.
   */
  public readonly lateArgs: IndirectStorageBufferAttribute = new IndirectStorageBufferAttribute(
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
  /**
   * The matrices as one node every static shader reads. Three names a buffer in a shader by its node, so a node per
   * material made every static shader's source unique: a pipeline per material, 974 of them on Pripyat.
   */
  public readonly modelColumns = storage(this.models, "vec4", STATIC_DRAW_CAPACITY * 4).toReadOnly();
  /** Each place of an instanced draw: its matrix, then its hemisphere terms in the fifth column. */
  public readonly places: StorageBufferAttribute = new StorageBufferAttribute(
    new Float32Array(STATIC_PLACE_CAPACITY * STATIC_PLACE_COLUMNS * 4),
    4
  );
  /** The places as one node every instanced static shader reads, for the reason `modelColumns` is one. */
  public readonly placeColumns = storage(
    this.places,
    "vec4",
    STATIC_PLACE_CAPACITY * STATIC_PLACE_COLUMNS
  ).toReadOnly();
  /** Each row's sphere in renderer space, a negative radius for a row testing nothing. */
  public readonly rowSpheres: StorageBufferAttribute = new StorageBufferAttribute(
    new Float32Array(STATIC_ROW_CAPACITY * 4).fill(-1),
    4
  );
  /** Each row's place, its draw's slot, where its draw's list starts, and the indices its draw takes. */
  public readonly rowTargets: StorageBufferAttribute = new StorageBufferAttribute(
    new Uint32Array(STATIC_ROW_CAPACITY * 4),
    4
  );
  /**
   * The places the instance culls kept, each instanced draw's from its first instance on: the first cull's in the
   * first half, the second's in the second.
   */
  public readonly visible: StorageBufferAttribute = new StorageBufferAttribute(
    new Uint32Array(STATIC_ROW_CAPACITY * 2),
    1
  );
  /** The list as one node every instanced static shader reads. */
  public readonly visiblePlaces = storage(this.visible, "uint", STATIC_ROW_CAPACITY * 2).toReadOnly();
  /** What the first cull decided for each slot, an `EStaticCullState`. */
  public readonly slotStates: StorageBufferAttribute = new StorageBufferAttribute(
    new Uint32Array(STATIC_DRAW_CAPACITY),
    1
  );
  /** What the first cull decided for each row. */
  public readonly rowStates: StorageBufferAttribute = new StorageBufferAttribute(
    new Uint32Array(STATIC_ROW_CAPACITY),
    1
  );
  /** The depth pyramid: each level the farthest depth under a texel of the last, four by four, packed level by level. */
  public readonly pyramid: StorageBufferAttribute = new StorageBufferAttribute(
    new Float32Array(STATIC_PYRAMID_CAPACITY),
    1
  );
  /** What occlusion tests project by, and where the pyramid's levels are. */
  public readonly occlusion: OcclusionUniforms = new OcclusionUniforms();
  /** What the last cull kept: draws, then indices, for the frame report. */
  public readonly counts: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(2), 1);
}
