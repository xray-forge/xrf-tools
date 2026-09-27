import { Maybe, Nullable } from "@xrf/types";
import { Box3, Material, Matrix4, Object3D, Scene, Sphere, Vector3 } from "three/webgpu";

import { IRendererPoolUse, IRendererStaticDrawReport } from "#/contract/renderer-report";
import { IRendererInstances } from "#/contract/scene/renderer-object";
import { ISurfaceMaterial } from "#/material/surface-material";
import { ISceneClusterRun } from "#/scene/geometry/scene-cluster-run";
import { SceneClusters } from "#/scene/geometry/scene-clusters";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { createSceneRoot } from "#/scene/object/scene-mesh";
import { PlainShadowCasters } from "#/scene/static/plain-shadow-casters";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticArenas } from "#/scene/static/static-arenas";
import { IStaticSlotBatches, StaticBatches } from "#/scene/static/static-batches";
import { StaticClusters } from "#/scene/static/static-clusters";
import { StaticCull } from "#/scene/static/static-cull";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { toGrownCapacity } from "#/scene/static/static-growth";
import { StaticLods } from "#/scene/static/static-lods";
import { StaticPlaces } from "#/scene/static/static-places";
import { IStaticPools } from "#/scene/static/static-pools";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";
import { IStaticUpcoming } from "#/scene/static/static-upcoming";
import {
  EStaticListSpace,
  EStaticPool,
  EStaticSlotKind,
  STATIC_NO_BAND,
  STATIC_NO_BATCH,
  STATIC_SHADOW_VIEWS,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

/** A run of a pool's elements a slot holds: where it starts, and how many. */
interface IRun {
  start: number;
  count: number;
}

/** What one slot holds of the pools besides its record. */
interface ISlotHold {
  clusters: Nullable<IRun>;
  /** A single draw's place. */
  place: Nullable<number>;
  /** An instanced draw's rows, a place of its object each. */
  rows: Nullable<IRun>;
}

/** The pools a run is handed out of, which grow on their own. */
type TRunPool = EStaticPool.PLACES | EStaticPool.ROWS | EStaticPool.LODS | EStaticPool.CLUSTERS;

/** What the objects still waiting to draw will take of each pool. */
type TStaticDemand = Record<EStaticPool.SLOTS | EStaticPool.PLACES | EStaticPool.ROWS | EStaticPool.CLUSTERS, number>;

/** A corner of a place's sphere, reused. */
const PLACE_CORNER: Vector3 = new Vector3();

/**
 * Everything drawing static draws: the slots each draw's record sits in, its clusters and places, the rows of the
 * instanced ones, the arenas their geometry is copied into, the batches drawing them, one object a material and arena,
 * and the cull on the GPU. A pool that runs out grows, once for everything the queue is known to bring; a draw is drawn
 * plainly only where the device's limit stops it.
 */
export class StaticDraws implements IStaticShadowCasters, IStaticPools {
  /** What culls the static draws on the GPU, which the frame dispatches before drawing them and again between. */
  public readonly cull: StaticCull;
  /** Where the batches' second draws stand, drawn into the G-buffer after the second cull. */
  public readonly late: Scene = createSceneRoot();
  /** What each shadow view draws of the casting batches that stand still, by the view's arguments. */
  public readonly stillShadowScenes: ReadonlyArray<Scene> = Array.from(
    { length: STATIC_SHADOW_VIEWS },
    createSceneRoot
  );
  /** And of those that sway with the wind. */
  public readonly swayingShadowScenes: ReadonlyArray<Scene> = Array.from(
    { length: STATIC_SHADOW_VIEWS },
    createSceneRoot
  );
  /** What each shadow view draws: every casting batch, the still and the swaying scene within it. */
  public readonly shadowScenes: ReadonlyArray<Scene> = this.stillShadowScenes.map((still: Scene, view: number) => {
    const scene: Scene = createSceneRoot();

    scene.add(still, this.swayingShadowScenes[view]);

    return scene;
  });
  /** What every cascade draws besides the batches: a twin of each part drawn plainly that casts. */
  public readonly plainCasters: PlainShadowCasters = new PlainShadowCasters();
  /** The impostors of clumps of trees, which the LOD cull decides between a clump and its impostor by. */
  public readonly lods: StaticLods;

  private readonly buffers: StaticDrawBuffers;
  private readonly pool: StaticDrawPool;
  private readonly places: StaticPlaces;
  private readonly clusters: StaticClusters;
  private readonly arenas: StaticArenas;
  private readonly batches: StaticBatches;
  private readonly holds: Map<number, ISlotHold> = new Map();
  private readonly toUpcoming: () => Iterable<IStaticUpcoming>;
  /** Times a static draw was refused room by the device's limit and drawn plainly instead. */
  private fallbacks: number = 0;

  /**
   * @param buffers - What every static draw reads.
   * @param scene - The scene of the pass drawing static draws, where the batches stand.
   * @param toUpcoming - What the objects still waiting to draw will take of the static draws.
   */
  public constructor(buffers: StaticDrawBuffers, scene: Object3D, toUpcoming: () => Iterable<IStaticUpcoming>) {
    this.buffers = buffers;
    this.toUpcoming = toUpcoming;
    this.pool = new StaticDrawPool(buffers);
    this.places = new StaticPlaces(buffers);
    this.lods = new StaticLods(buffers);
    this.clusters = new StaticClusters(buffers);
    this.batches = new StaticBatches(buffers, scene, this.late, this.stillShadowScenes, this.swayingShadowScenes);
    this.arenas = new StaticArenas(
      (arena: StaticArena) => this.batches.invalidateArena(arena),
      (arena: StaticArena) => this.batches.release(arena),
      () => [...toUpcoming()].map((upcoming: IStaticUpcoming) => upcoming.geometry),
      () => buffers.storageLimit,
      buffers.listEntries,
      buffers.clusterRangeWords
    );
    this.cull = new StaticCull(buffers, this, this.late);
  }

  /** Whether static draws are drawn at all: only on a device drawing an indirect draw's first instance. */
  public get isEnabled(): boolean {
    return this.pool.isEnabled;
  }

  public set isEnabled(isEnabled: boolean) {
    this.pool.isEnabled = isEnabled;
  }

  /** Bumped whenever anything a cull lists changes. */
  public get version(): number {
    return this.pool.version + this.places.version + this.lods.version + this.clusters.version + this.batches.version;
  }

  public get clusterExtent(): number {
    return this.clusters.extent;
  }

  public get rowExtent(): number {
    return this.places.rowExtent;
  }

  public get batchExtent(): number {
    return this.batches.extent;
  }

  public get lodExtent(): number {
    return this.lods.extent;
  }

  /** Queues every pool's changes for upload, and hands the arenas' replaced buffers on to be freed. */
  public flush(): void {
    this.pool.flush();
    this.places.flush();
    this.lods.flush();
    this.clusters.flush();
    this.batches.flush();
    this.buffers.retire(this.arenas.takeRetired());
  }

  /**
   * @param material - What every static draw's edges draw with; null to draw triangles.
   */
  public setWireframe(material: Nullable<Material>): void {
    this.batches.setWireframe(material);
    this.cull.setWireframe(material !== null);
  }

  /** Where what the shadow views draw changed, and what of it sways or moves. */
  public get shadowChanges(): StaticShadowChanges {
    return this.batches.shadowChanges;
  }

  /** Every material a batch draws. */
  public get materials(): Iterable<Material> {
    return this.batches.materials;
  }

  /** How full each pool is, how many draws fell back to the plain path, and what the last cull found occluded. */
  public get report(): IRendererStaticDrawReport {
    const { clusters, triangles, occludedClusters, occludedTriangles } = this.cull.kept;

    return {
      clusters: this.clusters.use,
      commands: this.batches.surfaceCount * 2,
      fallbacks: this.fallbacks,
      kept: { clusters, triangles },
      lists: {
        shadows: this.batches.listUse(EStaticListSpace.SHADOWS),
        surfaces: this.batches.listUse(EStaticListSpace.SURFACES),
      },
      lods: this.lods.use,
      occluded: { clusters: occludedClusters, triangles: occludedTriangles },
      places: this.places.placeUse,
      rows: this.places.rowUse,
      slots: { capacity: this.pool.capacity, used: this.pool.count },
    };
  }

  /**
   * @param geometry - A geometry.
   * @returns The arena its layout sits in, whose prototype its static draws' materials compile against; null for a
   *   layout no arena stores.
   */
  public toArena(geometry: SceneGeometry): Nullable<StaticArena> {
    return this.arenas.toArena(geometry);
  }

  /**
   * @param geometry - A geometry one more object draws statically.
   * @returns Where it sits, or null where no arena can hold it.
   */
  public acquire(geometry: SceneGeometry): Nullable<IStaticRange> {
    return this.arenas.acquire(geometry);
  }

  /**
   * @param geometry - A geometry one object fewer draws statically.
   */
  public release(geometry: SceneGeometry): void {
    this.arenas.release(geometry);
  }

  /**
   * @returns A slot for one static draw, the slots grown where every one is taken; null where static draws are off or
   *   the slots cannot grow past the device's limit.
   */
  public allocate(): Nullable<number> {
    if (!this.pool.isEnabled) {
      return null;
    }

    const slot: Nullable<number> = this.pool.allocate() ?? (this.growSlots() ? this.pool.allocate() : null);

    if (slot === null) {
      this.fallbacks += 1;
    } else {
      this.holds.set(slot, { clusters: null, place: null, rows: null });
    }

    return slot;
  }

  /**
   * A single static draw: its clusters where its matrix stands them, each culled by its own sphere.
   *
   * @param slot - The slot drawing.
   * @param surface - What draws it.
   * @param range - Where its geometry sits.
   * @param clusters - Its geometry's clusters.
   * @param run - The clusters it draws; none draws nothing.
   * @param bounds - What it spans in renderer space.
   * @param matrix - Where it stands.
   * @returns Whether it is drawn so; not where there is no room for it, and it has to be drawn plainly.
   */
  public draw(
    slot: number,
    surface: ISurfaceMaterial,
    range: IStaticRange,
    clusters: SceneClusters,
    run: ISceneClusterRun,
    bounds: Sphere,
    matrix: Matrix4
  ): boolean {
    const hold: ISlotHold = this.holds.get(slot) as ISlotHold;

    this.freeRun(EStaticPool.ROWS, hold, "rows");

    if (hold.place === null) {
      hold.place = this.allocateRun(EStaticPool.PLACES, 1);
    }

    const start: Nullable<number> = this.holdClusters(hold, run.count);

    if (hold.place === null || start === null) {
      return this.refuse(slot, false);
    }

    this.places.writePlace(hold.place, matrix);
    this.clusters.write(start, slot, clusters, run, range, matrix);

    const batches: Nullable<IStaticSlotBatches> = this.batches.put(
      slot,
      range.arena,
      surface,
      run.count,
      bounds.getBoundingBox(new Box3())
    );

    if (!batches) {
      return this.refuse(slot);
    }

    this.pool.write(
      slot,
      run.count ? EStaticSlotKind.SINGLE : EStaticSlotKind.NONE,
      { count: run.count, start },
      hold.place,
      batches.surface,
      batches.shadow
    );

    return true;
  }

  /**
   * @param count - Places an instanced object stands in.
   * @returns Where its places start, or null where there is no room and it has to be drawn plainly.
   */
  public allocatePlaces(count: number): Nullable<number> {
    return this.pool.isEnabled ? this.allocateRun(EStaticPool.PLACES, count) : null;
  }

  /**
   * @param start - Where an instanced object's places start.
   * @param instances - Its places, as the consumer put them.
   * @param placement - Its own matrix, which places every instance.
   * @param lodStart - Where the impostors its places belong to start, or null where they belong to none.
   */
  public writePlaces(
    start: number,
    instances: IRendererInstances,
    placement: Matrix4,
    lodStart: Nullable<number> = null
  ): void {
    this.places.writePlaces(start, instances, placement, lodStart);
  }

  /**
   * @param count - Impostors a set holds.
   * @returns Where they start, the pool grown where it has no room; null where the device's limit stops it.
   */
  public allocateLods(count: number): Nullable<number> {
    return this.allocateRun(EStaticPool.LODS, count);
  }

  /**
   * @param start - Where a run of places starts, free for another object.
   * @param count - Its length.
   */
  public freePlaces(start: number, count: number): void {
    this.places.freePlaces(start, count);
  }

  /**
   * An instanced static draw: its clusters in every place a row of it keeps.
   *
   * @param slot - The slot drawing.
   * @param surface - What draws it.
   * @param range - Where its geometry sits.
   * @param clusters - Its geometry's clusters.
   * @param run - The clusters it draws, in its mesh's own space; none draws nothing.
   * @param placeStart - Where its object's places start.
   * @param spheres - Each place's sphere in renderer space, four floats each, which its rows test.
   * @param lods - Each place's impostor as its row names it, or null where none stands in for any.
   * @param band - Which band of a progressive mesh it is (`toStaticBandWord`), `STATIC_NO_BAND` for one detail.
   * @returns Whether it is drawn so; not where there is no room for its rows.
   */
  public drawListed(
    slot: number,
    surface: ISurfaceMaterial,
    range: IStaticRange,
    clusters: SceneClusters,
    run: ISceneClusterRun,
    placeStart: number,
    spheres: Float32Array,
    lods: Nullable<Uint32Array> = null,
    band: number = STATIC_NO_BAND
  ): boolean {
    const hold: ISlotHold = this.holds.get(slot) as ISlotHold;
    const places: number = spheres.length / 4;

    this.freePlace(hold);

    if (hold.rows && hold.rows.count !== places) {
      this.freeRun(EStaticPool.ROWS, hold, "rows");
    }

    if (!hold.rows) {
      const rowStart: Nullable<number> = this.allocateRun(EStaticPool.ROWS, places);

      hold.rows = rowStart === null ? null : { count: places, start: rowStart };
    }

    const start: Nullable<number> = this.holdClusters(hold, run.count);

    if (!hold.rows || start === null) {
      return this.refuse(slot, false);
    }

    const activeSpheres: Float32Array = run.count ? spheres : new Float32Array(spheres.length).fill(-1);

    this.clusters.write(start, slot, clusters, run, range, null);
    this.places.writeRows(hold.rows.start, activeSpheres, placeStart, slot, lods, band);

    const batches: Nullable<IStaticSlotBatches> = this.batches.put(
      slot,
      range.arena,
      surface,
      run.count * places,
      StaticDraws.toPlacesBox(spheres),
      activeSpheres
    );

    if (!batches) {
      return this.refuse(slot);
    }

    this.pool.write(
      slot,
      run.count ? EStaticSlotKind.LISTED : EStaticSlotKind.NONE,
      { count: run.count, start },
      0,
      batches.surface,
      batches.shadow
    );

    return true;
  }

  /**
   * @param slot - A slot drawing nothing from now on.
   */
  public free(slot: number): void {
    const hold: Maybe<ISlotHold> = this.holds.get(slot);

    this.batches.withdraw(slot);

    if (hold) {
      this.freeRun(EStaticPool.ROWS, hold, "rows");
      this.freeRun(EStaticPool.CLUSTERS, hold, "clusters");
      this.freePlace(hold);
      this.holds.delete(slot);
    }

    this.pool.release(slot);
  }

  /**
   * @param key - A texture key whose samplers now sample another texture.
   */
  public invalidate(key: string): void {
    this.batches.invalidate(key);
  }

  public dispose(): void {
    this.batches.dispose();
    this.arenas.dispose();
    this.buffers.retire(this.arenas.takeRetired());
    this.cull.dispose();
  }

  /**
   * @param spheres - Each place's sphere, four floats each, a negative radius for none.
   * @returns The box every place spans.
   */
  private static toPlacesBox(spheres: Float32Array): Box3 {
    const box: Box3 = new Box3();

    for (let at = 0; at < spheres.length; at += 4) {
      const radius: number = spheres[at + 3];

      if (radius >= 0) {
        box.expandByPoint(PLACE_CORNER.set(spheres[at] - radius, spheres[at + 1] - radius, spheres[at + 2] - radius));
        box.expandByPoint(PLACE_CORNER.set(spheres[at] + radius, spheres[at + 1] + radius, spheres[at + 2] + radius));
      }
    }

    return box;
  }

  /**
   * Takes a slot out of what drew it, its record left drawing nothing, for it to be drawn plainly.
   *
   * @param slot - The slot.
   * @param isCounted - Whether this refusal counts as a fallback: not where the pool that refused counted it.
   * @returns Nothing drawn statically.
   */
  private refuse(slot: number, isCounted: boolean = true): boolean {
    const hold: Maybe<ISlotHold> = this.holds.get(slot);

    this.batches.withdraw(slot);

    if (hold) {
      this.freeRun(EStaticPool.ROWS, hold, "rows");
      this.freeRun(EStaticPool.CLUSTERS, hold, "clusters");
      this.freePlace(hold);
    }

    this.pool.write(slot, EStaticSlotKind.NONE, { count: 0, start: 0 }, 0, STATIC_NO_BATCH, STATIC_NO_BATCH);

    if (isCounted) {
      this.fallbacks += 1;
    }

    return false;
  }

  /** A slot's run of clusters of the length wanted: kept where it is that long, taken again otherwise. */
  private holdClusters(hold: ISlotHold, count: number): Nullable<number> {
    if (hold.clusters && hold.clusters.count !== count) {
      this.freeRun(EStaticPool.CLUSTERS, hold, "clusters");
    }

    if (!hold.clusters) {
      const start: Nullable<number> = this.allocateRun(EStaticPool.CLUSTERS, count);

      hold.clusters = start === null ? null : { count, start };
    }

    return hold.clusters?.start ?? null;
  }

  private freeRun(pool: EStaticPool.ROWS | EStaticPool.CLUSTERS, hold: ISlotHold, key: "rows" | "clusters"): void {
    const run: Nullable<IRun> = hold[key];

    if (!run) {
      return;
    }

    if (pool === EStaticPool.ROWS) {
      this.places.freeRows(run.start, run.count);
    } else {
      this.clusters.free(run.start, run.count);
    }

    hold[key] = null;
  }

  private freePlace(hold: ISlotHold): void {
    if (hold.place !== null) {
      this.places.freePlaces(hold.place, 1);
      hold.place = null;
    }
  }

  /** Grows the slots once for what the queue brings. */
  private growSlots(): boolean {
    const capacity: number = toGrownCapacity(
      this.pool.count,
      1 + this.toDemand()[EStaticPool.SLOTS],
      this.pool.capacity,
      this.buffers.initial(EStaticPool.SLOTS),
      this.buffers.limit(EStaticPool.SLOTS)
    );

    if (capacity <= this.pool.capacity) {
      return false;
    }

    this.buffers.grow(EStaticPool.SLOTS, capacity);
    this.batches.invalidateAll();

    return true;
  }

  /** @returns A run of the pool's, where it holds room for one, or null. */
  private takeRun(pool: TRunPool, count: number): Nullable<number> {
    switch (pool) {
      case EStaticPool.PLACES:
        return this.places.allocatePlaces(count);
      case EStaticPool.ROWS:
        return this.places.allocateRows(count);
      case EStaticPool.LODS:
        return this.lods.allocate(count);
      case EStaticPool.CLUSTERS:
        return this.clusters.allocate(count);
    }
  }

  /**
   * @returns A run of a pool, grown until it fits: first for what the queue brings, then past the whole run, where
   *   freed room lies in runs too short for it. Null where the device's limit stops it.
   */
  private allocateRun(pool: TRunPool, count: number): Nullable<number> {
    const limit: number = this.buffers.limit(pool);
    let start: Nullable<number> = this.takeRun(pool, count);
    let isFirst: boolean = true;

    while (start === null) {
      const use: IRendererPoolUse = this.toUse(pool);
      const capacity: number = toGrownCapacity(
        isFirst ? use.used : use.capacity,
        count + (isFirst && pool !== EStaticPool.LODS ? this.toDemand()[pool] : 0),
        use.capacity,
        this.buffers.initial(pool),
        limit
      );

      if (capacity <= use.capacity) {
        this.fallbacks += 1;

        return null;
      }

      switch (pool) {
        case EStaticPool.LODS:
          this.lods.grow(capacity);
          break;
        case EStaticPool.CLUSTERS:
          this.clusters.grow(capacity);
          break;
        default:
          this.places.grow(pool, capacity);
      }

      // Every material reading the grown buffer binds the new one once its batch records.
      this.batches.invalidateAll();
      isFirst = false;
      start = this.takeRun(pool, count);
    }

    return start;
  }

  private toUse(pool: TRunPool): IRendererPoolUse {
    switch (pool) {
      case EStaticPool.PLACES:
        return this.places.placeUse;
      case EStaticPool.ROWS:
        return this.places.rowUse;
      case EStaticPool.LODS:
        return this.lods.use;
      case EStaticPool.CLUSTERS:
        return this.clusters.use;
    }
  }

  /** What the objects still waiting to draw will take of each pool, besides what they hold already. */
  private toDemand(): TStaticDemand {
    const demand: TStaticDemand = {
      [EStaticPool.CLUSTERS]: 0,
      [EStaticPool.PLACES]: 0,
      [EStaticPool.ROWS]: 0,
      [EStaticPool.SLOTS]: 0,
    };

    for (const { sections, places, clusters } of this.toUpcoming()) {
      demand[EStaticPool.SLOTS] += sections;
      demand[EStaticPool.PLACES] += places || sections;
      demand[EStaticPool.ROWS] += places * sections;
      demand[EStaticPool.CLUSTERS] += clusters;
    }

    return demand;
  }
}
