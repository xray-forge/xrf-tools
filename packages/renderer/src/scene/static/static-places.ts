import { Nullable } from "@xrf/types";
import { BufferAttribute, Matrix4 } from "three/webgpu";

import { IRendererPoolUse } from "#/contract/renderer-report";
import {
  IRendererInstances,
  RENDERER_FLOATS_PER_INSTANCE,
  RENDERER_HEMI_FLOATS_PER_INSTANCE,
} from "#/contract/scene/renderer-object";
import { queueBufferUpload } from "#/scene/buffer-upload";
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

/** The elements written since the buffers last went up, as one span. */
interface IDirtySpan {
  first: number;
  last: number;
}

/**
 * Where instanced static draws stand and what the instance cull tests: places, a matrix and hemisphere terms each;
 * rows, a place of one draw each; and the list each draw's kept places are written to. Handed out in runs, uploaded as
 * one span a buffer.
 */
export class StaticPlaces {
  private readonly buffers: StaticDrawBuffers;
  private readonly places: RangeAllocator = new RangeAllocator();
  private readonly rows: RangeAllocator = new RangeAllocator();
  private readonly placeSpan: IDirtySpan = { first: Infinity, last: -1 };
  private readonly rowSpan: IDirtySpan = { first: Infinity, last: -1 };
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
    }

    StaticPlaces.touch(this.placeSpan, start, start + count - 1);
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
   * @param count - Rows wanted, and as many entries in the list of kept places.
   * @returns Where both start, or null where there is no room.
   */
  public allocateRows(count: number): Nullable<number> {
    return this.rows.allocate(count);
  }

  /**
   * Makes a run of rows test every place of one instanced draw, listing the kept ones from the run's own start.
   *
   * @param start - Where the rows start.
   * @param spheres - Each place's sphere in renderer space, four floats each.
   * @param placeStart - Where the places start.
   * @param slot - The draw's slot, whose instance count the rows count up.
   * @param indexCount - Indices the draw takes, for the report.
   * @param lods - Each row's impostor as the LOD cull reads it, or null where none stands in for any place.
   * @param band - The draw's band of a progressive mesh (`toStaticBandWord`), `STATIC_NO_BAND` for a draw of one
   * detail.
   */
  public writeRows(
    start: number,
    spheres: Float32Array,
    placeStart: number,
    slot: number,
    indexCount: number,
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
      targets[at + 2] = start;
      targets[at + 3] = indexCount;
    }

    StaticPlaces.touch(this.rowSpan, start, start + count - 1);
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
    StaticPlaces.touch(this.rowSpan, start, start + count - 1);
    this.currentVersion += 1;
  }

  /** Marks what changed since the last upload to go up with the next use of the buffers. */
  public flush(): void {
    StaticPlaces.upload(this.buffers.places, this.placeSpan, FLOATS_PER_PLACE);
    StaticPlaces.upload(this.buffers.rowSpheres, this.rowSpan, 4);
    StaticPlaces.upload(this.buffers.rowTargets, this.rowSpan, 4);
    StaticPlaces.upload(this.buffers.rowLods, this.rowSpan, 2);
    this.rowSpan.first = Infinity;
    this.rowSpan.last = -1;
    this.placeSpan.first = Infinity;
    this.placeSpan.last = -1;
  }

  private static touch(span: IDirtySpan, first: number, last: number): void {
    span.first = Math.min(span.first, first);
    span.last = Math.max(span.last, last);
  }

  private static upload(attribute: BufferAttribute, span: IDirtySpan, stride: number): void {
    if (span.last < span.first) {
      return;
    }

    queueBufferUpload(attribute, span.first * stride, (span.last - span.first + 1) * stride);
  }
}
