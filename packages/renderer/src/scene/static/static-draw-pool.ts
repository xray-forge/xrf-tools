import { Maybe, Nullable } from "@xrf/types";

import { DirtySpan } from "#/scene/dirty-span";
import { ISceneClusterRun } from "#/scene/geometry/scene-cluster-run";
import { STATIC_NO_BATCH, STATIC_SLOT_WORDS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { EStaticSlotKind } from "#/uniforms/static-slot-kind";
import { SURFACE_NO_ROW } from "#/uniforms/surface-table";

/**
 * Hands out the slots of the static draw buffers and writes what each draws: its clusters, its place and its batches,
 * uploading the slots changed in a frame as one span.
 */
export class StaticDrawPool {
  /** Whether static draws are drawn at all: only on a device drawing an indirect draw's first instance. */
  public isEnabled: boolean = false;

  private readonly buffers: StaticDrawBuffers;
  private readonly free: Array<number> = [];
  /** The slots free again, which a slot released twice is not given out twice by. */
  private readonly freed: Set<number> = new Set();
  /** Slots handed out at least once; the cull only reads below it. */
  private used: number = 0;
  /** The slots written since the buffers last went up. */
  private readonly dirty: DirtySpan = new DirtySpan();
  private currentVersion: number = 0;

  public constructor(buffers: StaticDrawBuffers) {
    this.buffers = buffers;
  }

  /** Bumped whenever a slot changes what it draws or where, so a cull knows to run again. */
  public get version(): number {
    return this.currentVersion;
  }

  /** Slots in use, a draw each. */
  public get count(): number {
    return this.used - this.free.length;
  }

  /** Slots the buffers hold. */
  public get capacity(): number {
    return this.buffers.capacity(EStaticPool.SLOTS);
  }

  /** Slots the cull has to look at: every slot ever handed out. */
  public get extent(): number {
    return this.used;
  }

  /**
   * @returns A slot, or null where static draws are off or every slot is taken, until the buffers grow.
   */
  public allocate(): Nullable<number> {
    if (!this.isEnabled) {
      return null;
    }

    const slot: Maybe<number> = this.free.pop();

    if (slot !== undefined) {
      this.freed.delete(slot);

      return slot;
    }

    if (this.used === this.capacity) {
      return null;
    }

    this.used += 1;

    return this.used - 1;
  }

  /**
   * @param slot - The slot drawing.
   * @param kind - What it draws.
   * @param clusters - Where its clusters sit in the clusters pool.
   * @param place - Where a single draw stands; nothing for an instanced one, whose rows name their places.
   * @param surfaceBatch - The batch drawing it into the G-buffer.
   * @param shadowBatch - The batch drawing it into the shadow views, `STATIC_NO_BATCH` for one casting nothing.
   * @param row - The surface table's row its surface reads, `SURFACE_NO_ROW` for one drawing by its own material.
   */
  public write(
    slot: number,
    kind: EStaticSlotKind,
    clusters: ISceneClusterRun,
    place: number,
    surfaceBatch: number,
    shadowBatch: number,
    row: number = SURFACE_NO_ROW
  ): void {
    (this.buffers.slots.array as Uint32Array).set(
      [clusters.start, clusters.count, place, kind, surfaceBatch, shadowBatch, row, 0],
      slot * STATIC_SLOT_WORDS
    );
    this.touch(slot);
  }

  /**
   * @param slot - A slot drawing nothing from now on, free for another draw; one free already stays as it is.
   */
  public release(slot: number): void {
    if (this.freed.has(slot) || slot >= this.used) {
      return;
    }

    (this.buffers.slots.array as Uint32Array).set(
      [0, 0, 0, EStaticSlotKind.NONE, STATIC_NO_BATCH, STATIC_NO_BATCH, SURFACE_NO_ROW, 0],
      slot * STATIC_SLOT_WORDS
    );
    this.free.push(slot);
    this.freed.add(slot);
    this.touch(slot);
  }

  /** Marks what changed since the last upload to go up with the next use of the buffers. */
  public flush(): void {
    this.dirty.upload(this.buffers.slots, STATIC_SLOT_WORDS);
    this.dirty.clear();
  }

  private touch(slot: number): void {
    this.dirty.touch(slot);
    this.currentVersion += 1;
  }
}
