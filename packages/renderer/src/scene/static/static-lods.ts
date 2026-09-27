import { Nullable } from "@xrf/types";

import { IRendererPoolUse } from "#/contract/renderer-report";
import {
  IRendererImpostors,
  RENDERER_IMPOSTOR_CORNER_FLOATS,
  RENDERER_IMPOSTOR_CORNERS,
  RENDERER_IMPOSTOR_FACETS,
} from "#/contract/scene/renderer-impostors";
import { DirtySpan } from "#/scene/dirty-span";
import { RangeAllocator } from "#/scene/static/range-allocator";
import { EStaticPool, STATIC_LOD_CORNER_COLUMNS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/**
 * The impostors of clumps of trees, one slot each, that the LOD cull decides between a clump and its impostor by:
 * handed out in runs a set each, uploaded as one span a buffer.
 */
export class StaticLods {
  private readonly buffers: StaticDrawBuffers;
  private readonly lods: RangeAllocator = new RangeAllocator();
  /** The slots written since the buffers last went up. */
  private readonly span: DirtySpan = new DirtySpan();
  private currentVersion: number = 0;

  public constructor(buffers: StaticDrawBuffers) {
    this.buffers = buffers;
    this.lods.grow(buffers.capacity(EStaticPool.LODS));
  }

  /** Bumped whenever an impostor is written or freed, so a cull knows to run again. */
  public get version(): number {
    return this.currentVersion;
  }

  /** Impostors handed out, against what the buffers hold. */
  public get use(): IRendererPoolUse {
    return { capacity: this.lods.capacity, used: this.lods.used };
  }

  /** Slots the LOD cull has to look at: up to the end of the last run handed out. */
  public get extent(): number {
    return this.lods.extent;
  }

  /**
   * @param count - Impostors wanted.
   * @returns Where they start, or null where there is no room until the pool grows.
   */
  public allocate(count: number): Nullable<number> {
    return this.lods.allocate(count);
  }

  /**
   * @param capacity - What the pool holds from now on, more than it did.
   */
  public grow(capacity: number): void {
    this.buffers.grow(EStaticPool.LODS, capacity);
    this.lods.grow(capacity);
    this.currentVersion += 1;
  }

  /**
   * Copies a set's impostors in, the corners reordered into the two columns each the shaders read.
   *
   * @param start - Where the run starts.
   * @param impostors - The set.
   */
  public write(start: number, impostors: IRendererImpostors): void {
    const count: number = impostors.factors.length;
    const corners = this.buffers.lodCorners.array as Float32Array;

    (this.buffers.lodSpheres.array as Float32Array).set(impostors.spheres, start * 4);
    (this.buffers.lodFactors.array as Float32Array).set(impostors.factors, start);
    (this.buffers.lodNormals.array as Float32Array).set(impostors.normals, start * RENDERER_IMPOSTOR_FACETS * 4);

    // The contract's corner is eight floats: position, u, v, hemi, sun, nothing. A column pair is (position, hemi)
    // and (u, v, sun, nothing), so a corner reads as two aligned vec4s.
    for (let corner = 0; corner < count * RENDERER_IMPOSTOR_CORNERS; corner += 1) {
      const from: number = corner * RENDERER_IMPOSTOR_CORNER_FLOATS;
      const to: number = (start * STATIC_LOD_CORNER_COLUMNS + corner * 2) * 4;
      const source: Float32Array = impostors.corners;

      corners.set([source[from], source[from + 1], source[from + 2], source[from + 5]], to);
      corners.set([source[from + 3], source[from + 4], source[from + 6], 0], to + 4);
    }

    this.span.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  /**
   * @param start - Where a run starts, holding no impostor from now on.
   * @param count - Its length.
   */
  public free(start: number, count: number): void {
    const spheres = this.buffers.lodSpheres.array as Float32Array;

    for (let index = 0; index < count; index += 1) {
      spheres[(start + index) * 4 + 3] = -1;
    }

    this.lods.release(start, count);
    this.span.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  /** Marks what changed since the last upload to go up with the next use of the buffers. */
  public flush(): void {
    const { span } = this;

    span.upload(this.buffers.lodSpheres, 4);
    span.upload(this.buffers.lodFactors, 1);
    span.upload(this.buffers.lodNormals, RENDERER_IMPOSTOR_FACETS * 4);
    span.upload(this.buffers.lodCorners, STATIC_LOD_CORNER_COLUMNS * 4);
    span.clear();
  }
}
