import { storage } from "three/tsl";
import {
  BufferAttribute,
  IndirectStorageBufferAttribute,
  StorageBufferAttribute,
  StorageBufferNode,
  TypedArray,
} from "three/webgpu";

import { DEFAULT_STORAGE_LIMIT } from "#/internals/renderer-backend";
import { LodUniforms } from "#/uniforms/lod-uniforms";
import { OcclusionUniforms } from "#/uniforms/occlusion-uniforms";

/** Unsigned integers one indirect draw takes: index count, instance count, first index, base vertex, first instance. */
export const STATIC_DRAW_ARGUMENTS: number = 5;

/** Columns one place takes: its matrix's four, then its hemisphere scale and offset. */
export const STATIC_PLACE_COLUMNS: number = 5;

/** Unsigned integers the cull counts into: kept draws and indices, then occluded draws, instances and indices. */
export const STATIC_CULL_COUNTS: number = 5;

/** Vec4 columns one impostor's corners take: two each, the position and hemisphere term, then the atlas and sun. */
export const STATIC_LOD_CORNER_COLUMNS: number = 64;

/** What a row names for its impostor where none stands in for its place. */
export const STATIC_NO_LOD: number = 0xffffffff;

/** The bit of a row's impostor saying the row is the impostor's own draw rather than a tree of its clump. */
export const STATIC_LOD_IMPOSTOR_ROW: number = 0x80000000;

/** What a row names for its band where its draw is of one detail, drawing whatever detail its place is at. */
export const STATIC_NO_BAND: number = 0xffffffff;

/**
 * A row's band word: which band of how many its draw is, of a mesh of how many windows. A row keeps a place only where
 * the window the place's detail picks falls in that band.
 *
 * @param band - The draw's band.
 * @param bands - Bands the mesh is drawn in.
 * @param windows - Windows the engine's table has.
 * @returns The word.
 */
export function toStaticBandWord(band: number, bands: number, windows: number): number {
  return (band | (bands << 8) | (windows << 16)) >>> 0;
}

/**
 * The pools of the static draw buffers, each grown on its own: slots a draw each, places an instance each, rows a
 * place of one instanced draw each, impostors of clumps of trees, and the depth pyramid's texels.
 */
export enum EStaticPool {
  SLOTS = "slots",
  PLACES = "places",
  ROWS = "rows",
  LODS = "lods",
  PYRAMID = "pyramid",
}

/** What the LOD cull decided a clump draws as, bits of its terms' fourth word. */
export enum EStaticLodState {
  /** Its trees, near enough to be drawn in full. */
  TREES = 1,
  /** Its impostor, far enough to be drawn in their place. */
  IMPOSTOR = 2,
}

/** What each pool holds before it first grows: more than any level measured puts in its resident sectors. */
export const INITIAL_STATIC_CAPACITY: Readonly<Record<EStaticPool, number>> = {
  [EStaticPool.SLOTS]: 1 << 16,
  [EStaticPool.PLACES]: 1 << 16,
  [EStaticPool.ROWS]: 1 << 17,
  [EStaticPool.LODS]: 1 << 13,
  // A drawing of 4096 by 4096.
  [EStaticPool.PYRAMID]: 1 << 20,
};

/** Bytes the widest buffer of each pool takes an element, which is what a storage buffer's limit caps the pool by. */
const ELEMENT_BYTES: Readonly<Record<EStaticPool, number>> = {
  [EStaticPool.SLOTS]: 64,
  [EStaticPool.PLACES]: STATIC_PLACE_COLUMNS * 16,
  [EStaticPool.ROWS]: 16,
  [EStaticPool.LODS]: STATIC_LOD_CORNER_COLUMNS * 16,
  [EStaticPool.PYRAMID]: 4,
};

/** What the first cull decided for a slot or row: out of view, drawn, or left for the second cull as occluded. */
export enum EStaticCullState {
  OUTSIDE = 0,
  DRAWN = 1,
  OCCLUDED = 2,
}

type TStorageAttribute = StorageBufferAttribute | IndirectStorageBufferAttribute;

/**
 * @returns An attribute like `attribute` holding `count` elements, the old ones copied in and the new ones `fill`.
 */
function toGrown<T extends TStorageAttribute>(attribute: T, count: number, fill: number = 0): T {
  const source: TypedArray = attribute.array as TypedArray;
  const array: TypedArray = new (source.constructor as new (length: number) => TypedArray)(count * attribute.itemSize);

  if (fill) {
    array.fill(fill, source.length);
  }

  array.set(source);

  return new (attribute.constructor as new (array: TypedArray, itemSize: number) => T)(array, attribute.itemSize);
}

/**
 * What every static draw reads and is drawn by, one slot a draw: the arguments a compute pass culls, the sphere it
 * culls by, and the matrix the draw's shader places it with. One set of buffers for every static draw, so no draw has
 * a uniform of its own to refresh. A single draw's first instance is its slot, which its shader reads back through
 * its arena's slot attribute (`EVertexAttribute.STATIC_SLOT`), every element of which holds its own number. An
 * instanced draw's first instance is where its list of kept places starts: a place per instance, whose matrix and
 * hemisphere terms its shader reads. Each pool grows by replacing its buffers; the nodes every material reads stay,
 * pointed at the new ones.
 */
export class StaticDrawBuffers {
  /** Bytes one storage buffer may hold, which the device says once it is open. */
  public storageLimit: number = DEFAULT_STORAGE_LIMIT;

  /** Indirect arguments of the first draw, the instance count of each written by the first cull on the GPU. */
  public args: IndirectStorageBufferAttribute;
  /**
   * The same arguments for the second draw, of what the first cull found occluded by the last frame's depth and the
   * second finds seen by this frame's: the instance counts are the second cull's, an instanced draw lists its places
   * in the second half of the list.
   */
  public lateArgs: IndirectStorageBufferAttribute;
  /** Each draw's sphere in renderer space, a negative radius for a slot drawing nothing. */
  public spheres: StorageBufferAttribute;
  /** Each draw's matrix, four columns. */
  public models: StorageBufferAttribute;
  /** What the first cull decided for each slot, an `EStaticCullState`. */
  public slotStates: StorageBufferAttribute;
  /** Each place of an instanced draw: its matrix, then its hemisphere terms in the fifth column. */
  public places: StorageBufferAttribute;
  /** Each row's sphere in renderer space, a negative radius for a row testing nothing. */
  public rowSpheres: StorageBufferAttribute;
  /** Each row's place, its draw's slot, where its draw's list starts, and the indices its draw takes. */
  public rowTargets: StorageBufferAttribute;
  /**
   * The places the instance culls kept, each instanced draw's from its first instance on: the first cull's in the
   * first half, the second's in the second.
   */
  public visible: StorageBufferAttribute;
  /** What the first cull decided for each row. */
  public rowStates: StorageBufferAttribute;
  /**
   * Each row's detail words: its impostor, `STATIC_NO_LOD` for none and `STATIC_LOD_IMPOSTOR_ROW` set on the
   * impostor's own draw, then its band (`toStaticBandWord`), `STATIC_NO_BAND` for none.
   */
  public rowLods: StorageBufferAttribute;
  /** Each impostor's sphere in renderer space, a negative radius for a slot holding none. */
  public lodSpheres: StorageBufferAttribute;
  /** Each impostor's `FLOD::lod_factor`. */
  public lodFactors: StorageBufferAttribute;
  /** Each impostor's eight facet normals. */
  public lodNormals: StorageBufferAttribute;
  /** Each impostor's 32 corners, two columns each. */
  public lodCorners: StorageBufferAttribute;
  /**
   * What the LOD cull decided for each impostor: its best facet, the next, the fade and blend bytes, and an
   * `EStaticLodState`.
   */
  public lodTerms: StorageBufferAttribute;
  /** The depth pyramid: each level the farthest depth under a texel of the last, four by four, packed level by level. */
  public pyramid: StorageBufferAttribute;

  /**
   * The matrices as one node every static shader reads. Three names a buffer in a shader by its node, so a node per
   * material made every static shader's source unique: a pipeline per material, 974 of them on Pripyat.
   */
  public readonly modelColumns: StorageBufferNode<"vec4">;
  /** The places as one node every instanced static shader reads, for the reason `modelColumns` is one. */
  public readonly placeColumns: StorageBufferNode<"vec4">;
  /** The list as one node every instanced static shader reads. */
  public readonly visiblePlaces: StorageBufferNode<"uint">;
  /** The impostors' spheres as one node every impostor shader reads. */
  public readonly lodSphereColumns: StorageBufferNode<"vec4">;
  /** Their corners as one node every impostor shader reads. */
  public readonly lodCornerColumns: StorageBufferNode<"vec4">;
  /** Their terms as one node every impostor shader reads. */
  public readonly lodTermColumns: StorageBufferNode<"uvec4">;

  /** What occlusion tests project by, and where the pyramid's levels are. */
  public readonly occlusion: OcclusionUniforms = new OcclusionUniforms();
  /** What the LOD cull decides a clump of trees by. */
  public readonly lod: LodUniforms = new LodUniforms();
  /** What the last cull counted, `STATIC_CULL_COUNTS` of them, for the frame report. */
  public readonly counts: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(STATIC_CULL_COUNTS), 1);

  private readonly initials: Readonly<Record<EStaticPool, number>>;
  private readonly capacities: Record<EStaticPool, number>;
  /** Buffers replaced by a growth, whose GPU buffers go once nothing binds them. */
  private retired: Array<BufferAttribute> = [];
  private currentLayout: number = 0;

  /**
   * @param initial - What each pool holds before it first grows, where not `INITIAL_STATIC_CAPACITY`.
   */
  public constructor(initial: Partial<Record<EStaticPool, number>> = {}) {
    const capacities: Record<EStaticPool, number> = { ...INITIAL_STATIC_CAPACITY, ...initial };
    const slots: number = capacities[EStaticPool.SLOTS];
    const rows: number = capacities[EStaticPool.ROWS];
    const lods: number = capacities[EStaticPool.LODS];

    this.initials = { ...capacities };
    this.capacities = capacities;
    this.args = new IndirectStorageBufferAttribute(
      new Uint32Array(slots * STATIC_DRAW_ARGUMENTS),
      STATIC_DRAW_ARGUMENTS
    );
    this.lateArgs = new IndirectStorageBufferAttribute(
      new Uint32Array(slots * STATIC_DRAW_ARGUMENTS),
      STATIC_DRAW_ARGUMENTS
    );
    this.spheres = new StorageBufferAttribute(new Float32Array(slots * 4).fill(-1), 4);
    this.models = new StorageBufferAttribute(new Float32Array(slots * 16), 4);
    this.slotStates = new StorageBufferAttribute(new Uint32Array(slots), 1);
    this.places = new StorageBufferAttribute(
      new Float32Array(capacities[EStaticPool.PLACES] * STATIC_PLACE_COLUMNS * 4),
      4
    );
    this.rowSpheres = new StorageBufferAttribute(new Float32Array(rows * 4).fill(-1), 4);
    this.rowTargets = new StorageBufferAttribute(new Uint32Array(rows * 4), 4);
    this.visible = new StorageBufferAttribute(new Uint32Array(rows * 2), 1);
    this.rowStates = new StorageBufferAttribute(new Uint32Array(rows), 1);
    this.rowLods = new StorageBufferAttribute(new Uint32Array(rows * 2).fill(STATIC_NO_LOD), 2);
    this.lodSpheres = new StorageBufferAttribute(new Float32Array(lods * 4).fill(-1), 4);
    this.lodFactors = new StorageBufferAttribute(new Float32Array(lods), 1);
    this.lodNormals = new StorageBufferAttribute(new Float32Array(lods * 8 * 4), 4);
    this.lodCorners = new StorageBufferAttribute(new Float32Array(lods * STATIC_LOD_CORNER_COLUMNS * 4), 4);
    this.lodTerms = new StorageBufferAttribute(new Uint32Array(lods * 4), 4);
    this.pyramid = new StorageBufferAttribute(new Float32Array(capacities[EStaticPool.PYRAMID]), 1);
    this.modelColumns = storage(this.models, "vec4", slots * 4).toReadOnly();
    this.placeColumns = storage(
      this.places,
      "vec4",
      capacities[EStaticPool.PLACES] * STATIC_PLACE_COLUMNS
    ).toReadOnly();
    this.visiblePlaces = storage(this.visible, "uint", rows * 2).toReadOnly();
    this.lodSphereColumns = storage(this.lodSpheres, "vec4", lods).toReadOnly();
    this.lodCornerColumns = storage(this.lodCorners, "vec4", lods * STATIC_LOD_CORNER_COLUMNS).toReadOnly();
    this.lodTermColumns = storage(this.lodTerms, "uvec4", lods).toReadOnly();
  }

  /** Bumped by every growth: a shader built over the buffers before it reads the replaced ones. */
  public get layout(): number {
    return this.currentLayout;
  }

  /**
   * @param pool - A pool.
   * @returns What it holds now.
   */
  public capacity(pool: EStaticPool): number {
    return this.capacities[pool];
  }

  /**
   * @param pool - A pool.
   * @returns What it held before it first grew, which it never grows to less than.
   */
  public initial(pool: EStaticPool): number {
    return this.initials[pool];
  }

  /**
   * @param pool - A pool.
   * @returns What it may grow to: as many elements as its widest buffer holds within the storage limit.
   */
  public limit(pool: EStaticPool): number {
    return Math.floor(this.storageLimit / ELEMENT_BYTES[pool]);
  }

  /**
   * Replaces a pool's buffers with ones holding `capacity` elements, everything written so far copied in and uploaded
   * whole with their next use.
   *
   * @param pool - The pool.
   * @param capacity - What it holds from now on, more than it did and within its limit.
   */
  public grow(pool: EStaticPool, capacity: number): void {
    switch (pool) {
      case EStaticPool.SLOTS:
        this.args = this.replace(this.args, capacity);
        this.lateArgs = this.replace(this.lateArgs, capacity);
        this.spheres = this.replace(this.spheres, capacity, -1);
        this.models = this.replace(this.models, capacity * 4);
        this.slotStates = this.replace(this.slotStates, capacity);
        this.modelColumns.value = this.models;
        break;

      case EStaticPool.PLACES:
        this.places = this.replace(this.places, capacity * STATIC_PLACE_COLUMNS);
        this.placeColumns.value = this.places;
        break;

      case EStaticPool.ROWS:
        this.rowSpheres = this.replace(this.rowSpheres, capacity, -1);
        this.rowTargets = this.replace(this.rowTargets, capacity);
        // Twice over, the halves apart: the second cull lists its places a row capacity on, which moves with it.
        this.visible = this.replace(this.visible, capacity * 2);
        this.rowStates = this.replace(this.rowStates, capacity);
        this.rowLods = this.replace(this.rowLods, capacity, STATIC_NO_LOD);
        this.visiblePlaces.value = this.visible;
        break;

      case EStaticPool.LODS:
        this.lodSpheres = this.replace(this.lodSpheres, capacity, -1);
        this.lodFactors = this.replace(this.lodFactors, capacity);
        this.lodNormals = this.replace(this.lodNormals, capacity * 8);
        this.lodCorners = this.replace(this.lodCorners, capacity * STATIC_LOD_CORNER_COLUMNS);
        this.lodTerms = this.replace(this.lodTerms, capacity);
        this.lodSphereColumns.value = this.lodSpheres;
        this.lodCornerColumns.value = this.lodCorners;
        this.lodTermColumns.value = this.lodTerms;
        break;

      case EStaticPool.PYRAMID:
        this.pyramid = this.replace(this.pyramid, capacity);
        break;
    }

    this.capacities[pool] = capacity;
    this.currentLayout += 1;
  }

  /** @returns The buffers replaced since the last call, for their GPU buffers to go. */
  public takeRetired(): Array<BufferAttribute> {
    const retired: Array<BufferAttribute> = this.retired;

    this.retired = [];

    return retired;
  }

  private replace<T extends TStorageAttribute>(attribute: T, count: number, fill: number = 0): T {
    const grown: T = toGrown(attribute, count, fill);

    this.retired.push(attribute);

    return grown;
  }
}
