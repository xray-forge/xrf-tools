import { Maybe, Nullable } from "@xrf/types";
import { Box3, Material, Object3D } from "three/webgpu";

import { IRendererPoolUse } from "#/contract/renderer-pool-use";
import { ISurfaceMaterial } from "#/material/surface-material";
import { EShadowCasterMotion } from "#/scene/static/shadow-caster-motion";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch } from "#/scene/static/static-batch";
import { TStaticBatchArguments } from "#/scene/static/static-batch-arguments";
import { StaticBundleChunks } from "#/scene/static/static-bundle-chunks";
import { toGrownCapacity } from "#/scene/static/static-growth";
import { StaticListRegions } from "#/scene/static/static-list-regions";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";
import { IStaticSlotBatches } from "#/scene/static/static-slot-batches";
import { STATIC_NO_BATCH, STATIC_SHADOW_VIEWS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticListSpace } from "#/uniforms/static-list-space";
import { EStaticPool } from "#/uniforms/static-pool";
import { EStaticView } from "#/uniforms/static-view";

/** One way of batching the slots: the list space and arguments its batches draw by, and the bundles they are in. */
interface IBatchGrouping {
  space: EStaticListSpace;
  phases: ReadonlyArray<TStaticBatchArguments>;
  chunks: StaticBundleChunks;
  /** Each arena's batches drawing, by material, and the idle ones kept for another material. */
  drawing: Map<StaticArena, Map<Material, StaticBatch>>;
  idle: Map<StaticArena, Array<StaticBatch>>;
  /** The batch drawing each slot. */
  slots: Map<number, StaticBatch>;
}

/**
 * The batches drawing every static draw, one an arena and material, each recorded in a chunk's bundles while it draws
 * any, and each with its region of its list space. Every slot is batched twice over: by its surface's own material,
 * drawn in the G-buffer, and, for a surface that casts, by its shadow material, drawn into each shadow view: apart for
 * what sways with the wind, so a light's face can keep what stands still and draw again only what moves. Every opaque
 * surface shares one shadow material, so its casters are a batch an arena, however many surfaces they are. While a
 * wireframe draws, every surface batch draws its own region as its clusters' edges, by one material but an
 * impostor's own. A batch records again over buffers a growth replaced once the batches next flush.
 */
export class StaticBatches {
  /** Where what the shadow views draw changed, and what of it sways or moves. */
  public readonly shadowChanges: StaticShadowChanges = new StaticShadowChanges();

  private readonly buffers: StaticDrawBuffers;
  private readonly regions: StaticListRegions;
  private readonly surfaces: IBatchGrouping;
  private readonly shadows: IBatchGrouping;
  private readonly swayingShadows: IBatchGrouping;
  private readonly wires: StaticBundleChunks;
  /** Every grouping, for what is done to each alike. */
  private readonly groupings: ReadonlyArray<IBatchGrouping>;
  /** Batch numbers handed out, and those free again. */
  private readonly free: Array<number> = [];
  private used: number = 0;
  /** The buffers' layout, and each arena's generation, the batches last recorded over. */
  private layout: number;
  private readonly generations: Map<StaticArena, number> = new Map();
  /** What a wireframe draws every surface with but an impostor, or null while none draws. */
  private wireMaterial: Nullable<Material> = null;
  private currentVersion: number = 0;

  /**
   * @param buffers - What every static draw reads.
   * @param early - Where a batch's first draw stands.
   * @param late - Where its draw by the second cull's list stands.
   * @param stillScenes - Where a shadow batch of what stands still draws, by each shadow view's arguments.
   * @param swayingScenes - Where one of what sways with the wind draws.
   */
  public constructor(
    buffers: StaticDrawBuffers,
    early: Object3D,
    late: Object3D,
    stillScenes: ReadonlyArray<Object3D>,
    swayingScenes: ReadonlyArray<Object3D>
  ) {
    function toShadowArgs(): Array<TStaticBatchArguments> {
      return Array.from(
        { length: STATIC_SHADOW_VIEWS },
        (_: unknown, view: number) => () => buffers.viewArgs[EStaticView.SHADOW + view]
      );
    }

    this.buffers = buffers;
    this.layout = buffers.layout;
    this.regions = new StaticListRegions(buffers);
    this.surfaces = StaticBatches.createGrouping(
      EStaticListSpace.SURFACES,
      [() => buffers.viewArgs[EStaticView.EARLY], () => buffers.viewArgs[EStaticView.LATE]],
      [early, late]
    );
    this.shadows = StaticBatches.createGrouping(EStaticListSpace.SHADOWS, toShadowArgs(), stillScenes);
    this.swayingShadows = StaticBatches.createGrouping(EStaticListSpace.SHADOWS, toShadowArgs(), swayingScenes);
    this.wires = new StaticBundleChunks([early, late]);
    this.wires.setShown(false);
    this.groupings = [this.surfaces, this.shadows, this.swayingShadows];
  }

  /** Bumped whenever a batch or its region changes, so every view culls again. */
  public get version(): number {
    return this.currentVersion + this.regions.version;
  }

  /** Surface batches drawing, each drawn once a view of the camera's two. */
  public get surfaceCount(): number {
    let count: number = 0;

    this.surfaces.drawing.forEach((batches: Map<Material, StaticBatch>) => (count += batches.size));

    return count;
  }

  /** Whether any surface batch draws over an arena that sways with the wind. */
  public get isSwaying(): boolean {
    for (const [arena, batches] of this.surfaces.drawing) {
      if (arena.isSwaying && batches.size) {
        return true;
      }
    }

    return false;
  }

  /** Batches the cull has to clear the arguments of: every number ever handed out. */
  public get extent(): number {
    return this.used;
  }

  /**
   * @param space - A list space.
   * @returns Entries its batches' regions hold, against what it holds.
   */
  public listUse(space: EStaticListSpace): IRendererPoolUse {
    return this.regions.use(space);
  }

  /**
   * @param space - A list space.
   * @returns Where its last region ends: no view lists an entry at or past it.
   */
  public listExtent(space: EStaticListSpace): number {
    return this.regions.extent(space);
  }

  /** Every material a batch draws in the G-buffer. */
  public get materials(): Iterable<Material> {
    return [...this.surfaces.drawing.values()].flatMap((batches: Map<Material, StaticBatch>) => [...batches.keys()]);
  }

  /**
   * @param slot - A slot drawing from now on in the batches of its arena and materials, and in no others. Its batches
   *   record again even where they drew the slot already.
   * @param arena - The arena its geometry sits in.
   * @param surface - What draws it.
   * @param entries - Entries it may list at once: its clusters, times its rows for an instanced one.
   * @param bounds - What it spans in renderer space, or null where it is not known.
   * @param spheres - Individual instance spheres for precise animated-caster queries, copied by the change tracker.
   * @returns Its batches, or null where a region could not be made for either and it has to be drawn plainly.
   */
  public put(
    slot: number,
    arena: StaticArena,
    surface: ISurfaceMaterial,
    entries: number,
    bounds: Nullable<Box3> = null,
    spheres: Nullable<Float32Array> = null
  ): Nullable<IStaticSlotBatches> {
    const shadows: IBatchGrouping = arena.isSwaying ? this.swayingShadows : this.shadows;

    // A slot now over an arena of the other kind casts from the other grouping no more.
    this.take(arena.isSwaying ? this.shadows : this.swayingShadows, slot);

    const drawn: Nullable<StaticBatch> = this.putIn(
      this.surfaces,
      slot,
      arena,
      surface.material,
      surface.keys,
      entries,
      surface.isImpostor
    );
    // Not for a slot its surface refused: its shadow region would go at once, the list the spaces share grown for it.
    const cast: Nullable<StaticBatch> =
      drawn && surface.shadow ? this.putIn(shadows, slot, arena, surface.shadow, surface.shadowKeys, entries) : null;

    if (!drawn || (surface.shadow && !cast)) {
      this.withdraw(slot);

      return null;
    }

    if (!surface.shadow) {
      this.take(shadows, slot);
    }

    if (this.wireMaterial) {
      this.wire(drawn);
    }

    this.shadowChanges.put(
      slot,
      bounds,
      Boolean(surface.shadow),
      arena.isSwaying ? EShadowCasterMotion.SWAYING : EShadowCasterMotion.STILL,
      spheres
    );

    return { shadow: cast?.id ?? STATIC_NO_BATCH, surface: drawn.id };
  }

  /**
   * @param slot - A slot no batch draws from now on.
   */
  public withdraw(slot: number): void {
    this.groupings.forEach((grouping: IBatchGrouping) => this.take(grouping, slot));
    this.shadowChanges.withdraw(slot);
  }

  /**
   * Draws every surface batch's clusters as their edges, or as their triangles again: by one material, or by an
   * impostor's own, which turns its quad to the camera.
   *
   * @param material - What a wireframe draws with, or null to draw the triangles.
   */
  public setWireframe(material: Nullable<Material>): void {
    if (material === this.wireMaterial) {
      return;
    }

    this.wireMaterial = material;

    if (material) {
      this.surfaces.drawing.forEach((batches: Map<Material, StaticBatch>) =>
        batches.forEach((batch: StaticBatch) => this.wire(batch))
      );
    }

    this.surfaces.chunks.setShown(!material);
    this.wires.setShown(Boolean(material));
  }

  /**
   * @param key - A texture key whose samplers now sample another texture: every batch sampling it records again.
   */
  public invalidate(key: string): void {
    for (const grouping of this.groupings) {
      StaticBatches.all(grouping).forEach((batch: StaticBatch) => batch.keys.includes(key) && batch.invalidate());
    }

    // A shadow changes where a caster cuts out by the texture, not where only its colour does.
    for (const grouping of [this.shadows, this.swayingShadows]) {
      grouping.slots.forEach((batch: StaticBatch, slot: number) => {
        if (batch.keys.includes(key)) {
          this.shadowChanges.touch(slot);
        }
      });
    }
  }

  /**
   * Has every batch record again over the buffers a growth replaced since the last flush, and marks what changed
   * since the last upload to go up with the next use of the buffers.
   */
  public flush(): void {
    if (this.layout !== this.buffers.layout) {
      this.layout = this.buffers.layout;
      // Every static material reads the grown buffers, and every batch draws by the arguments as they are now.
      this.groupings.forEach((grouping: IBatchGrouping) =>
        StaticBatches.all(grouping).forEach((batch: StaticBatch) => batch.refresh())
      );
    }

    for (const grouping of this.groupings) {
      grouping.drawing.forEach((_: Map<Material, StaticBatch>, arena: StaticArena) => {
        if (this.generations.get(arena) !== arena.generation) {
          this.generations.set(arena, arena.generation);
          // Every batch over an arena that grew binds its new buffers, in every grouping.
          this.groupings.forEach((it: IBatchGrouping) =>
            it.drawing.get(arena)?.forEach((batch: StaticBatch) => batch.invalidate())
          );
        }
      });
    }

    this.regions.flush();
  }

  /** Lets every batch go with the static draws; the arenas persist until then, so none goes before. */
  public dispose(): void {
    for (const grouping of this.groupings) {
      StaticBatches.all(grouping).forEach((batch: StaticBatch) => this.drop(grouping, batch));
      grouping.drawing.clear();
      grouping.idle.clear();
      grouping.chunks.clear();
      grouping.slots.clear();
    }

    this.generations.clear();
    this.wires.clear();
  }

  /** Draws a surface batch's wireframe: an impostor's by its own material, every other by the one wireframe material. */
  private wire(batch: StaticBatch): void {
    const material: Material = batch.isImpostor ? (batch.material as Material) : (this.wireMaterial as Material);

    batch.setWire([() => this.buffers.wireArgs[0], () => this.buffers.wireArgs[1]], material);
    this.wires.attach(batch, batch.wireMeshes);
  }

  /** Puts a slot in its grouping's batch of its arena and material, and gives the batch room for it. */
  private putIn(
    grouping: IBatchGrouping,
    slot: number,
    arena: StaticArena,
    material: Material,
    keys: ReadonlyArray<string>,
    entries: number,
    isImpostor: boolean = false
  ): Nullable<StaticBatch> {
    let batches: Maybe<Map<Material, StaticBatch>> = grouping.drawing.get(arena);

    if (!batches) {
      batches = new Map();
      grouping.drawing.set(arena, batches);
    }

    let batch: Maybe<StaticBatch> = batches.get(material);

    if (grouping.slots.get(slot) !== batch) {
      this.take(grouping, slot);
    }

    if (!batch) {
      const created: Nullable<StaticBatch> = this.takeBatch(grouping, arena);

      if (!created) {
        return null;
      }

      batch = created;
      batch.setMaterial(material, keys, isImpostor);
      batches.set(material, batch);
      grouping.chunks.attach(batch, batch.meshes);
    }

    batch.put(slot, entries);
    grouping.slots.set(slot, batch);
    batch.invalidate();
    this.currentVersion += 1;

    if (!this.regions.fit(batch)) {
      this.take(grouping, slot);

      return null;
    }

    return batch;
  }

  /** An idle batch of the arena's, or a new one, numbered; null where the device's limit stops the batches growing. */
  private takeBatch(grouping: IBatchGrouping, arena: StaticArena): Nullable<StaticBatch> {
    const idle: Maybe<StaticBatch> = grouping.idle.get(arena)?.pop();

    if (idle) {
      return idle;
    }

    const id: Nullable<number> = this.allocateId();

    return id === null ? null : new StaticBatch(arena, id, grouping.space, grouping.phases);
  }

  /** A batch number, the batches grown where every one is taken; null where the device's limit stops them. */
  private allocateId(): Nullable<number> {
    if (this.free.length) {
      return this.free.pop() as number;
    }

    const capacity: number = this.buffers.capacity(EStaticPool.BATCHES);

    if (this.used === capacity) {
      const grown: number = toGrownCapacity(
        this.used,
        1,
        capacity,
        this.buffers.initial(EStaticPool.BATCHES),
        this.buffers.limit(EStaticPool.BATCHES)
      );

      if (grown <= capacity) {
        return null;
      }

      this.buffers.grow(EStaticPool.BATCHES, grown);
    }

    this.used += 1;

    return this.used - 1;
  }

  /** Takes a slot out of its batch in a grouping, the batch going idle once it draws nothing. */
  private take(grouping: IBatchGrouping, slot: number): void {
    const batch: Maybe<StaticBatch> = grouping.slots.get(slot);

    if (!batch) {
      return;
    }

    batch.remove(slot);
    batch.invalidate();
    grouping.slots.delete(slot);
    this.currentVersion += 1;

    if (batch.isEmpty) {
      grouping.drawing.get(batch.arena)?.delete(batch.material as Material);
      batch.setMaterial(null);
      this.unbundle(grouping, batch);

      let idle: Maybe<Array<StaticBatch>> = grouping.idle.get(batch.arena);

      if (!idle) {
        idle = [];
        grouping.idle.set(batch.arena, idle);
      }

      idle.push(batch);
    }
  }

  /** Lets a batch go: out of its bundles, its region and its number free. */
  private drop(grouping: IBatchGrouping, batch: StaticBatch): void {
    this.unbundle(grouping, batch);
    batch.dispose();
    this.free.push(batch.id);
  }

  /** Takes a batch's meshes out of every bundle, and gives its region to another. */
  private unbundle(grouping: IBatchGrouping, batch: StaticBatch): void {
    grouping.chunks.detach(batch);
    this.wires.detach(batch);
    this.regions.release(batch);
  }

  private static createGrouping(
    space: EStaticListSpace,
    phases: ReadonlyArray<TStaticBatchArguments>,
    scenes: ReadonlyArray<Object3D>
  ): IBatchGrouping {
    return {
      chunks: new StaticBundleChunks(scenes),
      drawing: new Map(),
      idle: new Map(),
      phases,
      slots: new Map(),
      space,
    };
  }

  /** Every batch of a grouping, drawing and idle. */
  private static all(grouping: IBatchGrouping): Array<StaticBatch> {
    return [
      ...[...grouping.drawing.values()].flatMap((batches: Map<Material, StaticBatch>) => [...batches.values()]),
      ...[...grouping.idle.values()].flat(),
    ];
  }
}
