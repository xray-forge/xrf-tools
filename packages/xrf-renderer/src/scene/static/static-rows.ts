import { Nullable } from "@xrf/types";

import { DirtySpan } from "#/scene/dirty-span";
import { RangeAllocator } from "#/scene/static/range-allocator";
import { IStaticRunPool } from "#/scene/static/static-run-pool";
import { STATIC_NO_BAND, STATIC_NO_LOD, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";

/**
 * What the instance cull tests: rows, a place of one instanced draw each, its sphere, its place and slot, and its
 * impostor and band. Handed out in runs, uploaded as one span a buffer.
 */
export class StaticRows implements IStaticRunPool {
  public readonly kind: EStaticPool = EStaticPool.ROWS;

  private readonly buffers: StaticDrawBuffers;
  private readonly rows: RangeAllocator = new RangeAllocator();
  /** The rows written since the buffers last went up. */
  private readonly span: DirtySpan = new DirtySpan();
  private currentVersion: number = 0;

  public constructor(buffers: StaticDrawBuffers) {
    this.buffers = buffers;
    this.rows.grow(buffers.capacity(EStaticPool.ROWS));
  }

  public get capacity(): number {
    return this.rows.capacity;
  }

  public get used(): number {
    return this.rows.used;
  }

  /** Rows the instance culls have to look at: up to the end of the last run handed out. */
  public get extent(): number {
    return this.rows.extent;
  }

  /** Bumped whenever a row changes or the rows grow, so a cull knows to run again. */
  public get version(): number {
    return this.currentVersion;
  }

  public fits(count: number): boolean {
    return this.rows.fits(count);
  }

  public allocate(count: number): Nullable<number> {
    return this.rows.allocate(count);
  }

  public grow(capacity: number): void {
    this.buffers.grow(EStaticPool.ROWS, capacity);
    this.rows.grow(capacity);
    this.currentVersion += 1;
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
   *   detail.
   */
  public write(
    start: number,
    spheres: Float32Array,
    placeStart: number,
    slot: number,
    lods: Nullable<Uint32Array> = null,
    band: number = STATIC_NO_BAND
  ): void {
    const count: number = spheres.length / 4;
    const targets: Uint32Array = this.buffers.rowTargets.array as Uint32Array;
    const rowLods: Uint32Array = this.buffers.rowLods.array as Uint32Array;

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

    this.span.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  /** Leaves a run of rows testing nothing, free for another draw. */
  public free(start: number, count: number): void {
    const spheres: Float32Array = this.buffers.rowSpheres.array as Float32Array;

    for (let index = 0; index < count; index += 1) {
      spheres[(start + index) * 4 + 3] = -1;
    }

    this.rows.release(start, count);
    this.span.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  public flush(): void {
    this.span.upload(this.buffers.rowSpheres, 4);
    this.span.upload(this.buffers.rowTargets, 4);
    this.span.upload(this.buffers.rowLods, 2);
    this.span.clear();
  }
}
