import { Nullable } from "@xrf/types";
import { Matrix4 } from "three/webgpu";

import { IRendererPoolUse } from "#/contract/renderer-report";
import {
  IRendererInstances,
  RENDERER_FLOATS_PER_INSTANCE,
  RENDERER_HEMI_FLOATS_PER_INSTANCE,
} from "#/contract/scene/renderer-object";
import { DirtySpan } from "#/scene/dirty-span";
import { RangeAllocator } from "#/scene/static/range-allocator";
import {
  EStaticPool,
  STATIC_NO_BAND,
  STATIC_NO_LOD,
  STATIC_PLACE_COLUMNS,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

/** Floats one place takes in the places buffer. */
const FLOATS_PER_PLACE: number = STATIC_PLACE_COLUMNS * 4;

/**
 * Where static draws stand and what the instance cull tests: places, a matrix, hemisphere terms, an impostor and the
 * greatest scale each, a single draw's one and an instanced draw's one an instance; and rows, a place of one instanced
 * draw each. Handed out in runs, uploaded as one span a buffer.
 */
export class StaticPlaces {
  private readonly buffers: StaticDrawBuffers;
  private readonly places: RangeAllocator = new RangeAllocator();
  private readonly rows: RangeAllocator = new RangeAllocator();
  /** The places and the rows written since the buffers last went up. */
  private readonly placeSpan: DirtySpan = new DirtySpan();
  private readonly rowSpan: DirtySpan = new DirtySpan();
  private readonly matrix: Matrix4 = new Matrix4();
  private currentVersion: number = 0;

  public constructor(buffers: StaticDrawBuffers) {
    this.buffers = buffers;
    this.places.grow(buffers.capacity(EStaticPool.PLACES));
    this.rows.grow(buffers.capacity(EStaticPool.ROWS));
  }

  /** Places handed out, against what the buffers hold. */
  public get placeUse(): IRendererPoolUse {
    return { capacity: this.places.capacity, used: this.places.used };
  }

  /** Rows handed out, against what the buffers hold. */
  public get rowUse(): IRendererPoolUse {
    return { capacity: this.rows.capacity, used: this.rows.used };
  }

  /** Rows the instance culls have to look at: up to the end of the last run handed out. */
  public get rowExtent(): number {
    return this.rows.extent;
  }

  /**
   * Grows the places or the rows, their buffers and the runs handed out of them.
   *
   * @param pool - The places or the rows.
   * @param capacity - What it holds from now on, more than it did.
   */
  public grow(pool: EStaticPool.PLACES | EStaticPool.ROWS, capacity: number): void {
    this.buffers.grow(pool, capacity);
    (pool === EStaticPool.PLACES ? this.places : this.rows).grow(capacity);
    this.currentVersion += 1;
  }

  /** Bumped whenever a row or place changes, so a cull knows to run again. */
  public get version(): number {
    return this.currentVersion;
  }

  /**
   * @param count - Places wanted.
   * @returns Where they start, or null where there is no room.
   */
  public allocatePlaces(count: number): Nullable<number> {
    return this.places.allocate(count);
  }

  /**
   * @param start - Where an object's places start.
   * @param instances - Its places, as the consumer put them.
   * @param placement - What places every instance: the object's own matrix.
   * @param lodStart - Where the impostors its places belong to start, or null where they belong to none.
   */
  public writePlaces(
    start: number,
    instances: IRendererInstances,
    placement: Matrix4,
    lodStart: Nullable<number> = null
  ): void {
    const impostors: Nullable<Int32Array> = lodStart === null ? null : (instances.impostors?.indices ?? null);

    const count: number = instances.transforms.length / RENDERER_FLOATS_PER_INSTANCE;
    const places = this.buffers.places.array as Float32Array;

    for (let index = 0; index < count; index += 1) {
      const at: number = (start + index) * FLOATS_PER_PLACE;

      this.matrix.fromArray(instances.transforms, index * RENDERER_FLOATS_PER_INSTANCE).premultiply(placement);
      places.set(this.matrix.elements, at);
      // Instances without terms of their own leave the vertex hemi as it is.
      places[at + 16] = instances.hemi ? instances.hemi[index * RENDERER_HEMI_FLOATS_PER_INSTANCE] : 1;
      places[at + 17] = instances.hemi ? instances.hemi[index * RENDERER_HEMI_FLOATS_PER_INSTANCE + 1] : 0;
      // The impostor a place's draw is, which an impostor shader reads its facets by; -1 for none.
      places[at + 18] = impostors && impostors[index] >= 0 ? (lodStart as number) + impostors[index] : -1;
      // What a cluster's sphere, in its mesh's own space, is scaled by where the place stands it.
      places[at + 19] = this.matrix.getMaxScaleOnAxis();
    }

    this.placeSpan.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  /**
   * @param at - A single draw's place.
   * @param matrix - Where it stands.
   */
  public writePlace(at: number, matrix: Matrix4): void {
    const places = this.buffers.places.array as Float32Array;
    const first: number = at * FLOATS_PER_PLACE;

    places.set(matrix.elements, first);
    places.set([1, 0, -1, matrix.getMaxScaleOnAxis()], first + 16);
    this.placeSpan.touch(at);
    this.currentVersion += 1;
  }

  /**
   * @param start - Where a run of places starts.
   * @param count - Its length.
   */
  public freePlaces(start: number, count: number): void {
    this.places.release(start, count);
  }

  /**
   * @param count - Rows wanted.
   * @returns Where they start, or null where there is no room.
   */
  public allocateRows(count: number): Nullable<number> {
    return this.rows.allocate(count);
  }

  /**
   * Makes a run of rows test every place of one instanced draw, each kept one standing the draw's clusters there.
   *
   * @param start - Where the rows start.
   * @param spheres - Each place's sphere in renderer space, four floats each.
   * @param placeStart - Where the places start.
   * @param slot - The draw's slot, whose clusters a kept row stands.
   * @param lods - Each row's impostor as the LOD cull reads it, or null where none stands in for any place.
   * @param band - The draw's band of a progressive mesh (`toStaticBandWord`), `STATIC_NO_BAND` for a draw of one
   * detail.
   */
  public writeRows(
    start: number,
    spheres: Float32Array,
    placeStart: number,
    slot: number,
    lods: Nullable<Uint32Array> = null,
    band: number = STATIC_NO_BAND
  ): void {
    const count: number = spheres.length / 4;
    const targets = this.buffers.rowTargets.array as Uint32Array;
    const rowLods = this.buffers.rowLods.array as Uint32Array;

    (this.buffers.rowSpheres.array as Float32Array).set(spheres, start * 4);

    for (let index = 0; index < count; index += 1) {
      const at: number = (start + index) * 4;

      rowLods[(start + index) * 2] = lods ? lods[index] : STATIC_NO_LOD;
      rowLods[(start + index) * 2 + 1] = band;

      targets[at] = placeStart + index;
      targets[at + 1] = slot;
      targets[at + 2] = 0;
      targets[at + 3] = 0;
    }

    this.rowSpan.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  /**
   * @param start - Where a run of rows starts, testing nothing from now on.
   * @param count - Its length.
   */
  public freeRows(start: number, count: number): void {
    const spheres = this.buffers.rowSpheres.array as Float32Array;

    for (let index = 0; index < count; index += 1) {
      spheres[(start + index) * 4 + 3] = -1;
    }

    this.rows.release(start, count);
    this.rowSpan.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  /** Marks what changed since the last upload to go up with the next use of the buffers. */
  public flush(): void {
    this.placeSpan.upload(this.buffers.places, FLOATS_PER_PLACE);
    this.rowSpan.upload(this.buffers.rowSpheres, 4);
    this.rowSpan.upload(this.buffers.rowTargets, 4);
    this.rowSpan.upload(this.buffers.rowLods, 2);
    this.rowSpan.clear();
    this.placeSpan.clear();
  }
}
