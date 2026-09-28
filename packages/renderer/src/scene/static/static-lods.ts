import { Nullable } from "@xrf/types";

import {
  IRendererImpostors,
  RENDERER_IMPOSTOR_CORNER_FLOATS,
  RENDERER_IMPOSTOR_CORNERS,
  RENDERER_IMPOSTOR_FACETS,
} from "#/contract/scene/renderer-impostors";
import { DirtySpan } from "#/scene/dirty-span";
import { RangeAllocator } from "#/scene/static/range-allocator";
import { IStaticRunPool } from "#/scene/static/static-run-pool";
import { STATIC_LOD_CORNER_COLUMNS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";

/**
 * The impostors of clumps of trees, one slot each, that the LOD cull decides between a clump and its impostor by:
 * handed out in runs a set each, uploaded as one span a buffer.
 */
export class StaticLods implements IStaticRunPool {
  public readonly kind: EStaticPool = EStaticPool.LODS;

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

  public get capacity(): number {
    return this.lods.capacity;
  }

  public get used(): number {
    return this.lods.used;
  }

  /** Slots the LOD cull has to look at: up to the end of the last run handed out. */
  public get extent(): number {
    return this.lods.extent;
  }

  public fits(count: number): boolean {
    return this.lods.fits(count);
  }

  public allocate(count: number): Nullable<number> {
    return this.lods.allocate(count);
  }

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
    const corners: Float32Array = this.buffers.lodCorners.array as Float32Array;

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

  /** Leaves a run holding no impostor, for the LOD cull to pass over. */
  public free(start: number, count: number): void {
    const spheres: Float32Array = this.buffers.lodSpheres.array as Float32Array;

    for (let index = 0; index < count; index += 1) {
      spheres[(start + index) * 4 + 3] = -1;
    }

    this.lods.release(start, count);
    this.span.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  public flush(): void {
    const { span } = this;

    span.upload(this.buffers.lodSpheres, 4);
    span.upload(this.buffers.lodFactors, 1);
    span.upload(this.buffers.lodNormals, RENDERER_IMPOSTOR_FACETS * 4);
    span.upload(this.buffers.lodCorners, STATIC_LOD_CORNER_COLUMNS * 4);
    span.clear();
  }
}
