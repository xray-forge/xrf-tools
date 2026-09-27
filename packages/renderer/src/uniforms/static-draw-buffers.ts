import { storage } from "three/tsl";
import {
  BufferAttribute,
  IndirectStorageBufferAttribute,
  StorageBufferAttribute,
  StorageBufferNode,
  TypedArray,
} from "three/webgpu";

import { RENDERER_MAX_SHADOW_CASCADES } from "#/contract/renderer-features";
import { RENDERER_CLUSTER_TRIANGLES } from "#/contract/scene/renderer-geometry";
import { DEFAULT_STORAGE_LIMIT } from "#/internals/renderer-backend";
import { LIGHT_SHADOW_FACE_BUDGET } from "#/uniforms/lights-uniforms";
import { LodUniforms } from "#/uniforms/lod-uniforms";
import { OcclusionUniforms } from "#/uniforms/occlusion-uniforms";

/** Unsigned integers one indexed indirect draw takes: index count, instance count, first index, base vertex, first instance. */
export const STATIC_DRAW_ARGUMENTS: number = 5;

/** Bytes one indexed indirect draw's arguments take. */
export const STATIC_DRAW_ARGUMENT_BYTES: number = STATIC_DRAW_ARGUMENTS * Uint32Array.BYTES_PER_ELEMENT;

/** Unsigned integers one batch's draw takes: vertex count, instance count, first vertex, first instance. */
export const STATIC_BATCH_ARGUMENTS: number = 4;

/** Bytes one batch's draw takes, which a batch's offset into a view's arguments counts in. */
export const STATIC_BATCH_ARGUMENT_BYTES: number = STATIC_BATCH_ARGUMENTS * Uint32Array.BYTES_PER_ELEMENT;

/** Vertices a batch draws a cluster with: its triangles, three a triangle, those past its own falling on its last. */
export const STATIC_CLUSTER_VERTICES: number = RENDERER_CLUSTER_TRIANGLES * 3;

/** Vertices a wireframe draws a cluster with: each triangle's three edges, two ends each. */
export const STATIC_CLUSTER_WIRE_VERTICES: number = RENDERER_CLUSTER_TRIANGLES * 6;

/** Columns one place takes: its matrix's four, then its hemisphere scale and offset, impostor and greatest scale. */
export const STATIC_PLACE_COLUMNS: number = 5;

/** Unsigned integers one slot's record takes, in two words of four: its clusters and place, then its batches and row. */
export const STATIC_SLOT_WORDS: number = 8;

/**
 * Shadow views a static draw is culled for besides the camera's: each sun cascade, then a reusable slot for each
 * local-light face scheduled in one frame. The slots are bounded by the execution budget, not the resident lights.
 */
export const STATIC_SHADOW_VIEWS: number = RENDERER_MAX_SHADOW_CASCADES + LIGHT_SHADOW_FACE_BUDGET;

/** The first local-light face slot, after the sun cascades. A batch uses consecutive slots from here. */
export const STATIC_LIGHT_VIEW_START: number = RENDERER_MAX_SHADOW_CASCADES;

/** The camera's views, then each shadow view's: what the lists of kept clusters and the batches' arguments are per. */
export enum EStaticView {
  /** The first phase: what the last frame's depth does not hide. */
  EARLY = 0,
  /** The second: what the first's depth no longer hides of what the last frame's did. */
  LATE = 1,
  /** The first shadow view; shadow view `n` is `SHADOW + n`. */
  SHADOW = 2,
}

/** Views a static draw is culled for. */
export const STATIC_VIEWS: number = EStaticView.SHADOW + STATIC_SHADOW_VIEWS;

/** The list space a batch's region is in: the camera's batches, or the shadow views'. */
export enum EStaticListSpace {
  SURFACES = 0,
  SHADOWS = 1,
}

/** What a slot names for a batch where none of that grouping draws it. */
export const STATIC_NO_BATCH: number = 0xffffffff;

/** Unsigned integers the cull counts into: kept clusters and triangles, then occluded clusters and triangles. */
export const STATIC_CULL_COUNTS: number = 4;

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

/** What a slot draws, a flag of its record's fourth word. */
export enum EStaticSlotKind {
  /** Nothing: a slot free or drawing no index. */
  NONE = 0,
  /** Its clusters once, where its place puts them, each tested by its own sphere. */
  SINGLE = 1,
  /** Its clusters in every place a row of it keeps. */
  LISTED = 2,
}

/**
 * The pools of the static draw buffers, each grown on its own: slots a draw each, places an instance each, rows a
 * place of one instanced draw each, impostors of clumps of trees, clusters, batches, the lists a view's kept clusters
 * are written to, and the depth pyramid's texels.
 */
export enum EStaticPool {
  SLOTS = "slots",
  PLACES = "places",
  ROWS = "rows",
  LODS = "lods",
  CLUSTERS = "clusters",
  BATCHES = "batches",
  /** Entries of the camera's batches' regions, a view each of the camera's two. */
  SURFACE_LIST = "surfaceList",
  /** Entries of the shadow batches' regions, a view each of the shadow views. */
  SHADOW_LIST = "shadowList",
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
  [EStaticPool.SLOTS]: 1 << 14,
  [EStaticPool.PLACES]: 1 << 15,
  [EStaticPool.ROWS]: 1 << 15,
  [EStaticPool.LODS]: 1 << 13,
  [EStaticPool.CLUSTERS]: 1 << 16,
  [EStaticPool.BATCHES]: 1 << 11,
  [EStaticPool.SURFACE_LIST]: 1 << 18,
  [EStaticPool.SHADOW_LIST]: 1 << 18,
  // A drawing of 4096 by 4096.
  [EStaticPool.PYRAMID]: 1 << 20,
};

/** Bytes the widest buffer of each pool takes an element, which is what a storage buffer's limit caps the pool by. */
const ELEMENT_BYTES: Readonly<Record<EStaticPool, number>> = {
  [EStaticPool.SLOTS]: STATIC_SLOT_WORDS * 4,
  [EStaticPool.PLACES]: STATIC_PLACE_COLUMNS * 16,
  [EStaticPool.ROWS]: 16,
  [EStaticPool.LODS]: STATIC_LOD_CORNER_COLUMNS * 16,
  [EStaticPool.CLUSTERS]: 16,
  [EStaticPool.BATCHES]: STATIC_BATCH_ARGUMENT_BYTES,
  // The list is two regions of the surfaces' and one a shadow view of the shadows', eight bytes an entry.
  [EStaticPool.SURFACE_LIST]: 8 * 2,
  [EStaticPool.SHADOW_LIST]: 8 * STATIC_SHADOW_VIEWS,
  [EStaticPool.PYRAMID]: 4,
};

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

  array.set(source.length > array.length ? source.subarray(0, array.length) : source);

  return new (attribute.constructor as new (array: TypedArray, itemSize: number) => T)(array, attribute.itemSize);
}

/**
 * What every static draw reads and is drawn by. A static draw is a slot: its clusters, the place its single draw
 * stands in or the rows its places are tested by, and the batches drawing it. The cull tests every cluster of what it
 * keeps and lists the ones it keeps too, with their place, in their batch's region of the view's list; a batch draws
 * its region by one indirect draw a view, a cluster an instance. One set of buffers for every static draw, so no draw
 * has a uniform of its own to refresh. Each pool grows by replacing its buffers; the nodes every material reads stay,
 * pointed at the new ones.
 */
export class StaticDrawBuffers {
  /** Bytes one storage buffer may hold, which the device says once it is open. */
  public storageLimit: number = DEFAULT_STORAGE_LIMIT;

  /**
   * Each slot's record: its first cluster, its clusters, its place, its `EStaticSlotKind`; then its surface batch, its
   * shadow batch, the row of the surface table its surface reads (`SURFACE_NO_ROW` for one drawing by a material of its
   * own), and nothing.
   */
  public slots: StorageBufferAttribute;
  /** Each place of a static draw: its matrix, then its hemisphere terms, impostor and greatest scale. */
  public places: StorageBufferAttribute;
  /** Each row's sphere in renderer space, a negative radius for a row testing nothing. */
  public rowSpheres: StorageBufferAttribute;
  /** Each row's place and its draw's slot, then nothing. */
  public rowTargets: StorageBufferAttribute;
  /**
   * Each row's detail words: its impostor, `STATIC_NO_LOD` for none and `STATIC_LOD_IMPOSTOR_ROW` set on the
   * impostor's own draw, then its band (`toStaticBandWord`), `STATIC_NO_BAND` for none.
   */
  public rowLods: StorageBufferAttribute;
  /** Each cluster's first index and base vertex in its arena, its triangles, and its slot. */
  public clusterRanges: StorageBufferAttribute;
  /** Each cluster's sphere: in renderer space for a single draw's, in its mesh's own for an instanced one's. */
  public clusterSpheres: StorageBufferAttribute;
  /** Each batch's region: where it starts in its list space, how many entries it holds, and its `EStaticListSpace`. */
  public batchRegions: StorageBufferAttribute;
  /** Each view's arguments, a batch's draw each: the cull clears and counts them, a region's first instance. */
  public viewArgs: Array<IndirectStorageBufferAttribute>;
  /** The camera's two views' arguments over line segments, while a wireframe draws: its count doubled. */
  public wireArgs: Array<IndirectStorageBufferAttribute>;
  /**
   * The kept clusters of every view, each a cluster and its place: the camera's first view, its second, then each
   * shadow view, each view as long as its list space.
   */
  public lists: StorageBufferAttribute;
  /** What the first cull left for the second: clusters the last frame's depth hid, and the place each stands in. */
  public candidates: StorageBufferAttribute;
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
   * The places as one node every static shader reads. Three names a buffer in a shader by its node, so a node per
   * material made every static shader's source unique: a pipeline per material.
   */
  public readonly placeColumns: StorageBufferNode<"vec4">;
  /** The slots' records as one node every shared surface shader reads its row by, two elements a slot. */
  public readonly slotWords: StorageBufferNode<"uvec4">;
  /** The clusters' ranges as one node every static shader reads. */
  public readonly clusterRangeWords: StorageBufferNode<"uvec4">;
  /** The lists as one node every static shader reads. */
  public readonly listEntries: StorageBufferNode<"uvec2">;
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
  /** How many candidates the first cull left, which the second cull runs over. */
  public readonly candidateCount: StorageBufferAttribute = new StorageBufferAttribute(new Uint32Array(1), 1);

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
    const rows: number = capacities[EStaticPool.ROWS];
    const lods: number = capacities[EStaticPool.LODS];
    const batches: number = capacities[EStaticPool.BATCHES];

    this.initials = { ...capacities };
    this.capacities = capacities;
    this.slots = new StorageBufferAttribute(new Uint32Array(capacities[EStaticPool.SLOTS] * STATIC_SLOT_WORDS), 4);
    this.places = new StorageBufferAttribute(
      new Float32Array(capacities[EStaticPool.PLACES] * STATIC_PLACE_COLUMNS * 4),
      4
    );
    this.rowSpheres = new StorageBufferAttribute(new Float32Array(rows * 4).fill(-1), 4);
    this.rowTargets = new StorageBufferAttribute(new Uint32Array(rows * 4), 4);
    this.rowLods = new StorageBufferAttribute(new Uint32Array(rows * 2).fill(STATIC_NO_LOD), 2);
    this.clusterRanges = new StorageBufferAttribute(new Uint32Array(capacities[EStaticPool.CLUSTERS] * 4), 4);
    this.clusterSpheres = new StorageBufferAttribute(new Float32Array(capacities[EStaticPool.CLUSTERS] * 4), 4);
    this.batchRegions = new StorageBufferAttribute(new Uint32Array(batches * 4), 4);
    this.viewArgs = Array.from({ length: STATIC_VIEWS }, () => StaticDrawBuffers.createArgs(batches));
    this.wireArgs = [StaticDrawBuffers.createArgs(batches), StaticDrawBuffers.createArgs(batches)];
    this.lists = new StorageBufferAttribute(new Uint32Array(this.toListLength() * 2), 2);
    this.candidates = new StorageBufferAttribute(new Uint32Array(capacities[EStaticPool.SURFACE_LIST] * 2), 2);
    this.lodSpheres = new StorageBufferAttribute(new Float32Array(lods * 4).fill(-1), 4);
    this.lodFactors = new StorageBufferAttribute(new Float32Array(lods), 1);
    this.lodNormals = new StorageBufferAttribute(new Float32Array(lods * 8 * 4), 4);
    this.lodCorners = new StorageBufferAttribute(new Float32Array(lods * STATIC_LOD_CORNER_COLUMNS * 4), 4);
    this.lodTerms = new StorageBufferAttribute(new Uint32Array(lods * 4), 4);
    this.pyramid = new StorageBufferAttribute(new Float32Array(capacities[EStaticPool.PYRAMID]), 1);
    this.placeColumns = storage(
      this.places,
      "vec4",
      capacities[EStaticPool.PLACES] * STATIC_PLACE_COLUMNS
    ).toReadOnly();
    this.slotWords = storage(this.slots, "uvec4", capacities[EStaticPool.SLOTS] * (STATIC_SLOT_WORDS / 4)).toReadOnly();
    this.clusterRangeWords = storage(this.clusterRanges, "uvec4", capacities[EStaticPool.CLUSTERS]).toReadOnly();
    this.listEntries = storage(this.lists, "uvec2", this.toListLength()).toReadOnly();
    this.lodSphereColumns = storage(this.lodSpheres, "vec4", lods).toReadOnly();
    this.lodCornerColumns = storage(this.lodCorners, "vec4", lods * STATIC_LOD_CORNER_COLUMNS).toReadOnly();
    this.lodTermColumns = storage(this.lodTerms, "uvec4", lods).toReadOnly();
  }

  /** Bumped by every growth: a shader built over the buffers before it reads the replaced ones. */
  public get layout(): number {
    return this.currentLayout;
  }

  /**
   * @param view - A view.
   * @returns Where its list starts: the camera's two views a surface list space each, then a shadow one each.
   */
  public toListBase(view: number): number {
    const surfaces: number = this.capacities[EStaticPool.SURFACE_LIST];

    return view < EStaticView.SHADOW
      ? view * surfaces
      : 2 * surfaces + (view - EStaticView.SHADOW) * this.capacities[EStaticPool.SHADOW_LIST];
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
   * whole with their next use. A list space's growth moves every view after it, so the lists start again empty.
   *
   * @param pool - The pool.
   * @param capacity - What it holds from now on, more than it did and within its limit.
   */
  public grow(pool: EStaticPool, capacity: number): void {
    switch (pool) {
      case EStaticPool.SLOTS:
        this.slots = this.replace(this.slots, capacity * (STATIC_SLOT_WORDS / 4));
        this.slotWords.value = this.slots;
        break;

      case EStaticPool.PLACES:
        this.places = this.replace(this.places, capacity * STATIC_PLACE_COLUMNS);
        this.placeColumns.value = this.places;
        break;

      case EStaticPool.ROWS:
        this.rowSpheres = this.replace(this.rowSpheres, capacity, -1);
        this.rowTargets = this.replace(this.rowTargets, capacity);
        this.rowLods = this.replace(this.rowLods, capacity, STATIC_NO_LOD);
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

      case EStaticPool.CLUSTERS:
        this.clusterRanges = this.replace(this.clusterRanges, capacity);
        this.clusterSpheres = this.replace(this.clusterSpheres, capacity);
        this.clusterRangeWords.value = this.clusterRanges;
        break;

      case EStaticPool.BATCHES:
        this.batchRegions = this.replace(this.batchRegions, capacity);
        this.viewArgs = this.viewArgs.map((args) => this.replaceArgs(args, capacity));
        this.wireArgs = this.wireArgs.map((args) => this.replaceArgs(args, capacity));
        break;

      case EStaticPool.SURFACE_LIST:
      case EStaticPool.SHADOW_LIST:
        break;

      case EStaticPool.PYRAMID:
        this.pyramid = this.replace(this.pyramid, capacity);
        break;
    }

    this.capacities[pool] = capacity;

    if (pool === EStaticPool.SURFACE_LIST || pool === EStaticPool.SHADOW_LIST) {
      this.retired.push(this.lists, this.candidates);
      this.lists = new StorageBufferAttribute(new Uint32Array(this.toListLength() * 2), 2);
      this.candidates = new StorageBufferAttribute(new Uint32Array(this.capacities[EStaticPool.SURFACE_LIST] * 2), 2);
      this.listEntries.value = this.lists;
    }

    this.currentLayout += 1;
  }

  /**
   * @param attributes - Buffers of what draws static draws, given up, to go with the pools' own once nothing binds
   *   them.
   */
  public retire(attributes: Iterable<BufferAttribute>): void {
    this.retired.push(...attributes);
  }

  /** @returns The buffers replaced since the last call, for their GPU buffers to go. */
  public takeRetired(): Array<BufferAttribute> {
    const retired: Array<BufferAttribute> = this.retired;

    this.retired = [];

    return retired;
  }

  /** Entries every view's list together takes. */
  private toListLength(): number {
    return this.toListBase(STATIC_VIEWS);
  }

  private replace<T extends TStorageAttribute>(attribute: T, count: number, fill: number = 0): T {
    const grown: T = toGrown(attribute, count, fill);

    this.retired.push(attribute);

    return grown;
  }

  /** New arguments for a view, which the cull writes whole each time it runs, so nothing of the old is kept. */
  private replaceArgs(args: IndirectStorageBufferAttribute, batches: number): IndirectStorageBufferAttribute {
    this.retired.push(args);

    return StaticDrawBuffers.createArgs(batches);
  }

  private static createArgs(batches: number): IndirectStorageBufferAttribute {
    return new IndirectStorageBufferAttribute(
      new Uint32Array(batches * STATIC_BATCH_ARGUMENTS),
      STATIC_BATCH_ARGUMENTS
    );
  }
}
