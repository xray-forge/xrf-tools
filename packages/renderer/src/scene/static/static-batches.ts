import { Maybe } from "@xrf/types";
import { Material, Object3D } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch } from "#/scene/static/static-batch";
import { StaticBundleChunks } from "#/scene/static/static-bundle-chunks";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";

/** An arena's batches of one kind: the ones drawing, by material, and the idle ones kept for the next material. */
interface IArenaBatches {
  drawing: Map<Material, StaticBatch>;
  idle: Array<StaticBatch>;
}

/**
 * The batches drawing every static draw, one an arena, kind and material, each recorded in a chunk's bundles while it
 * draws any.
 */
export class StaticBatches {
  private readonly pool: StaticDrawPool;
  private readonly chunks: StaticBundleChunks;
  private readonly arenas: Map<StaticArena, Record<EStaticDrawKind, IArenaBatches>> = new Map();
  /** The batch drawing each slot. */
  private readonly drawing: Map<number, StaticBatch> = new Map();

  /**
   * @param pool - The slots the batches draw.
   * @param scene - Where a batch drawing anything stands.
   * @param late - Where its draw by the second cull's arguments stands.
   */
  public constructor(pool: StaticDrawPool, scene: Object3D, late: Object3D) {
    this.pool = pool;
    this.chunks = new StaticBundleChunks(scene, late);
  }

  /** Every material a batch draws. */
  public get materials(): Iterable<Material> {
    return this.all().flatMap((batches: IArenaBatches) => [...batches.drawing.keys()]);
  }

  /**
   * @param slot - A slot drawing from now on in the batch of its arena and material, and in no other. Its batch
   *   records again even where it drew the slot already: only a recording uploads the matrices its draws read, since
   *   a replay refreshes nothing of the batch's own.
   * @param arena - The arena its geometry sits in.
   * @param kind - The kind of static draw it is.
   * @param surface - What draws it.
   */
  public put(slot: number, arena: StaticArena, kind: EStaticDrawKind, surface: ISurfaceMaterial): void {
    const batches: IArenaBatches = this.toArenaBatches(arena, kind);
    let batch: Maybe<StaticBatch> = batches.drawing.get(surface.material);

    if (batch && this.drawing.get(slot) === batch) {
      batch.invalidate();

      return;
    }

    this.withdraw(slot);

    if (!batch) {
      batch = batches.idle.pop() ?? new StaticBatch(arena, kind, this.pool);
      batch.setSurface(surface);
      batches.drawing.set(surface.material, batch);
      this.chunks.attach(batch);
    }

    batch.add(slot);
    this.drawing.set(slot, batch);
  }

  /**
   * @param slot - A slot no batch draws from now on.
   */
  public withdraw(slot: number): void {
    const batch: Maybe<StaticBatch> = this.drawing.get(slot);

    if (!batch) {
      return;
    }

    batch.remove(slot);
    this.drawing.delete(slot);

    if (batch.isEmpty) {
      const batches: IArenaBatches = this.toArenaBatches(batch.arena, batch.kind);

      batches.drawing.delete((batch.surface as ISurfaceMaterial).material);
      batch.setSurface(null);
      this.chunks.detach(batch);
      batches.idle.push(batch);
    }
  }

  /**
   * @param key - A texture key whose samplers now sample another texture: every batch sampling it records again.
   */
  public invalidate(key: string): void {
    for (const { drawing } of this.all()) {
      drawing.forEach((batch: StaticBatch) => batch.surface?.keys.includes(key) && batch.invalidate());
    }
  }

  /** Has every batch record again, for storage buffers its shaders read that were replaced by ones that grew. */
  public invalidateAll(): void {
    this.all().forEach(({ drawing }) => drawing.forEach((batch: StaticBatch) => batch.invalidate()));
  }

  /**
   * @param arena - An arena whose buffers may have been replaced, which every batch over it then draws.
   */
  public refresh(arena: StaticArena): void {
    for (const batches of Object.values(this.arenas.get(arena) ?? {})) {
      batches.drawing.forEach((batch: StaticBatch) => batch.refresh());
      batches.idle.forEach((batch: StaticBatch) => batch.refresh());
    }
  }

  /**
   * @param arena - An arena going, which no batch draws any more.
   */
  public release(arena: StaticArena): void {
    for (const batches of Object.values(this.arenas.get(arena) ?? {})) {
      batches.drawing.forEach((batch: StaticBatch) => {
        this.chunks.detach(batch);
        batch.dispose();
      });
      batches.idle.forEach((batch: StaticBatch) => batch.dispose());
    }

    this.arenas.delete(arena);
  }

  public dispose(): void {
    [...this.arenas.keys()].forEach((arena: StaticArena) => this.release(arena));
    this.chunks.clear();
    this.drawing.clear();
  }

  private toArenaBatches(arena: StaticArena, kind: EStaticDrawKind): IArenaBatches {
    let batches: Maybe<Record<EStaticDrawKind, IArenaBatches>> = this.arenas.get(arena);

    if (!batches) {
      batches = {
        [EStaticDrawKind.LISTED]: { drawing: new Map(), idle: [] },
        [EStaticDrawKind.SINGLE]: { drawing: new Map(), idle: [] },
      };
      this.arenas.set(arena, batches);
    }

    return batches[kind];
  }

  /** Every arena's batches of every kind. */
  private all(): Array<IArenaBatches> {
    return [...this.arenas.values()].flatMap((batches) => Object.values(batches));
  }
}
