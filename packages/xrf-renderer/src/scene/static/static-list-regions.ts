import { Nullable } from "@xrf/types";

import { IRendererPoolUse } from "#/contract/renderer-pool-use";
import { DirtySpan } from "#/scene/dirty-span";
import { RangeAllocator } from "#/scene/static/range-allocator";
import { StaticBatch } from "#/scene/static/static-batch";
import { allocateGrowing, STATIC_HEADROOM } from "#/scene/static/static-growth";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticListSpace } from "#/uniforms/static-list-space";
import { EStaticPool } from "#/uniforms/static-pool";

/** The pool each list space is. */
const SPACE_POOLS: Readonly<Record<EStaticListSpace, EStaticPool.SURFACE_LIST | EStaticPool.SHADOW_LIST>> = {
  [EStaticListSpace.SURFACES]: EStaticPool.SURFACE_LIST,
  [EStaticListSpace.SHADOWS]: EStaticPool.SHADOW_LIST,
};

/**
 * Where each batch lists what a view keeps of it: a region of its list space, as long as every entry its slots may
 * list at once (every cluster at its full detail, in every place), so no cull ever overflows one and nothing is read
 * back. A region is given room to spare, and moved when its batch outgrows it; a space grows when no free run is long
 * enough, and every view's list moves with it.
 */
export class StaticListRegions {
  private readonly buffers: StaticDrawBuffers;
  private readonly spaces: Record<EStaticListSpace, RangeAllocator> = {
    [EStaticListSpace.SURFACES]: new RangeAllocator(),
    [EStaticListSpace.SHADOWS]: new RangeAllocator(),
  };
  private readonly span: DirtySpan = new DirtySpan();
  private currentVersion: number = 0;

  public constructor(buffers: StaticDrawBuffers) {
    this.buffers = buffers;
    this.spaces[EStaticListSpace.SURFACES].grow(buffers.capacity(EStaticPool.SURFACE_LIST));
    this.spaces[EStaticListSpace.SHADOWS].grow(buffers.capacity(EStaticPool.SHADOW_LIST));
  }

  /** Bumped whenever a region moves, so every view culls again into it. */
  public get version(): number {
    return this.currentVersion;
  }

  /**
   * @param space - A list space.
   * @returns Entries its regions hold, against what it holds.
   */
  public use(space: EStaticListSpace): IRendererPoolUse {
    return { capacity: this.spaces[space].capacity, used: this.spaces[space].used };
  }

  /**
   * @param space - A list space.
   * @returns Where its last region ends: no view lists an entry at or past it.
   */
  public extent(space: EStaticListSpace): number {
    return this.spaces[space].extent;
  }

  /**
   * Gives a batch a region holding what it may list, moving it where it outgrew the one it had.
   *
   * @param batch - A batch whose slots changed.
   * @returns Whether the batch's region holds its demand; not where the space cannot grow to hold it, the region it
   *   had kept.
   */
  public fit(batch: StaticBatch): boolean {
    if (batch.region && batch.region.capacity >= batch.demand) {
      return true;
    }

    const capacity: number = Math.max(1, Math.ceil(batch.demand * STATIC_HEADROOM));
    const start: Nullable<number> = this.allocate(batch.space, capacity);

    if (start === null) {
      return false;
    }

    this.release(batch);
    batch.region = { capacity, start };
    this.write(batch.id, start, capacity, batch.space);

    return true;
  }

  /**
   * @param batch - A batch going idle or away, whose region another can take.
   */
  public release(batch: StaticBatch): void {
    if (batch.region) {
      this.spaces[batch.space].release(batch.region.start, batch.region.capacity);
      batch.region = null;
      this.write(batch.id, 0, 0, batch.space);
    }
  }

  /** Marks what changed since the last upload to go up with the next use of the buffers. */
  public flush(): void {
    this.span.upload(this.buffers.batchRegions, 4);
    this.span.clear();
  }

  /** A run of a space, the space grown once where none fits, or null where the device's limit says it never will. */
  private allocate(space: EStaticListSpace, count: number): Nullable<number> {
    const allocator: RangeAllocator = this.spaces[space];
    const pool: EStaticPool.SURFACE_LIST | EStaticPool.SHADOW_LIST = SPACE_POOLS[space];

    return allocateGrowing(
      allocator,
      count,
      this.buffers.initial(pool),
      this.buffers.limit(pool),
      (capacity: number) => {
        this.buffers.grow(pool, capacity);
        allocator.grow(capacity);
      }
    );
  }

  private write(batch: number, start: number, capacity: number, space: EStaticListSpace): void {
    (this.buffers.batchRegions.array as Uint32Array).set([start, capacity, space, 0], batch * 4);
    this.span.touch(batch);
    this.currentVersion += 1;
  }
}
