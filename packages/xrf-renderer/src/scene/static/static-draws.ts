import { Maybe, Nullable } from "@xrf/types";
import { Box3, Material, Matrix4, Object3D, Scene, Sphere, Vector3 } from "three/webgpu";

import { IRendererPoolUse } from "#/contract/renderer-pool-use";
import { IRendererStaticDrawReport } from "#/contract/renderer-static-draw-report";
import { IRendererImpostors } from "#/contract/scene/renderer-impostors";
import { IRendererInstances } from "#/contract/scene/renderer-instances";
import { ISurfaceMaterial } from "#/material/surface-material";
import { ISceneClusterRun } from "#/scene/geometry/scene-cluster-run";
import { SceneClusters } from "#/scene/geometry/scene-clusters";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { createSceneRoot } from "#/scene/object/scene-mesh";
import { PlainShadowCasters } from "#/scene/static/plain-shadow-casters";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticArenas } from "#/scene/static/static-arenas";
import { StaticBatches } from "#/scene/static/static-batches";
import { StaticClusters } from "#/scene/static/static-clusters";
import { StaticCull } from "#/scene/static/static-cull";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { allocateGrowing, toGrownCapacity } from "#/scene/static/static-growth";
import { StaticLods } from "#/scene/static/static-lods";
import { StaticPlaces } from "#/scene/static/static-places";
import { IStaticPools } from "#/scene/static/static-pools";
import { IStaticRange } from "#/scene/static/static-range";
import { StaticRows } from "#/scene/static/static-rows";
import { StaticRunPool } from "#/scene/static/static-run-pool";
import { IStaticRuns } from "#/scene/static/static-runs";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";
import { IStaticSlotBatches } from "#/scene/static/static-slot-batches";
import { StaticSlotHolds } from "#/scene/static/static-slot-holds";
import { IStaticUpcoming } from "#/scene/static/static-upcoming";
import {
  STATIC_NO_BAND,
  STATIC_NO_BATCH,
  STATIC_SHADOW_VIEWS,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";
import { EStaticListSpace } from "#/uniforms/static-list-space";
import { EStaticPool } from "#/uniforms/static-pool";
import { EStaticSlotKind } from "#/uniforms/static-slot-kind";

/** What one object still waiting to draw will take of each pool it is known to take of. */
const UPCOMING_DEMAND: Readonly<Partial<Record<EStaticPool, (upcoming: IStaticUpcoming) => number>>> = {
  [EStaticPool.SLOTS]: ({ sections }: IStaticUpcoming) => sections,
  [EStaticPool.PLACES]: ({ places, sections }: IStaticUpcoming) => places || sections,
  [EStaticPool.ROWS]: ({ places, sections }: IStaticUpcoming) => places * sections,
  [EStaticPool.CLUSTERS]: ({ clusters }: IStaticUpcoming) => clusters,
};

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

  private readonly buffers: StaticDrawBuffers;
  private readonly pool: StaticDrawPool;
  private readonly places: StaticPlaces;
  private readonly rows: StaticRows;
  /** The impostors of clumps of trees, which the LOD cull decides between a clump and its impostor by. */
  private readonly lods: StaticLods;
  private readonly clusters: StaticClusters;
  private readonly holds: StaticSlotHolds;
  private readonly arenas: StaticArenas;
  private readonly batches: StaticBatches;
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
    this.rows = new StaticRows(buffers);
    this.lods = new StaticLods(buffers);
    this.clusters = new StaticClusters(buffers);
    this.holds = new StaticSlotHolds((pool: StaticRunPool, count: number) => this.allocateRun(pool, count));
    this.batches = new StaticBatches(buffers, scene, this.late, this.stillShadowScenes, this.swayingShadowScenes);
    this.arenas = new StaticArenas(buffers, () =>
      [...toUpcoming()].map((upcoming: IStaticUpcoming) => upcoming.geometry)
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

  /** Bumped whenever anything a cull lists changes, or a buffer anything draws by is replaced. */
  public get version(): number {
    return (
      this.pool.version +
      this.places.version +
      this.rows.version +
      this.lods.version +
      this.clusters.version +
      this.batches.version +
      this.buffers.layout +
      this.arenas.generation
    );
  }

  public get clusterExtent(): number {
    return this.clusters.extent;
  }

  public get rowExtent(): number {
    return this.rows.extent;
  }

  public get batchExtent(): number {
    return this.batches.extent;
  }

  public get lodExtent(): number {
    return this.lods.extent;
  }

  public get candidateExtent(): number {
    return this.batches.listExtent(EStaticListSpace.SURFACES);
  }

  /** Queues every pool's changes for upload, the batches recorded again over whatever grew. */
  public flush(): void {
    this.pool.flush();
    this.places.flush();
    this.rows.flush();
    this.lods.flush();
    this.clusters.flush();
    this.batches.flush();
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
      clusters: StaticDraws.toUse(this.clusters),
      commands: this.batches.surfaceCount * 2,
      fallbacks: this.fallbacks,
      kept: { clusters, triangles },
      lists: {
        shadows: this.batches.listUse(EStaticListSpace.SHADOWS),
        surfaces: this.batches.listUse(EStaticListSpace.SURFACES),
      },
      lods: StaticDraws.toUse(this.lods),
      occluded: { clusters: occludedClusters, triangles: occludedTriangles },
      places: StaticDraws.toUse(this.places),
      rows: StaticDraws.toUse(this.rows),
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
    this.holds.drop(slot, this.rows);

    const place: Nullable<number> = this.holds.hold(slot, this.places, 1);
    const start: Nullable<number> = this.holds.hold(slot, this.clusters, run.count);

    if (place === null || start === null) {
      return this.refuse(slot, false);
    }

    this.places.writePlace(place, matrix);
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
      place,
      batches.surface,
      batches.shadow,
      surface.row
    );

    return true;
  }

  /**
   * @param count - Places an instanced object stands in.
   * @returns Where its places start, or null where there is no room and it has to be drawn plainly.
   */
  public allocatePlaces(count: number): Nullable<number> {
    return this.pool.isEnabled ? this.allocateRun(this.places, count) : null;
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
   * @param start - Where a run of places starts, free for another object.
   * @param count - Its length.
   */
  public freePlaces(start: number, count: number): void {
    this.places.free(start, count);
  }

  /**
   * @param count - Impostors a set holds.
   * @returns Where they start, the pool grown where it has no room; null where the device's limit stops it.
   */
  public allocateLods(count: number): Nullable<number> {
    return this.allocateRun(this.lods, count);
  }

  /**
   * @param start - Where a set's impostors start.
   * @param impostors - The set.
   */
  public writeLods(start: number, impostors: IRendererImpostors): void {
    this.lods.write(start, impostors);
  }

  /**
   * @param start - Where a set's impostors start, holding none from now on.
   * @param count - Its length.
   */
  public freeLods(start: number, count: number): void {
    this.lods.free(start, count);
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
    const places: number = spheres.length / 4;

    this.holds.drop(slot, this.places);

    const rowStart: Nullable<number> = this.holds.hold(slot, this.rows, places);
    const start: Nullable<number> = this.holds.hold(slot, this.clusters, run.count);

    if (rowStart === null || start === null) {
      return this.refuse(slot, false);
    }

    const activeSpheres: Float32Array = run.count ? spheres : new Float32Array(spheres.length).fill(-1);

    this.clusters.write(start, slot, clusters, run, range, null);
    this.rows.write(rowStart, activeSpheres, placeStart, slot, lods, band);

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
      batches.shadow,
      surface.row
    );

    return true;
  }

  /**
   * @param slot - A slot drawing nothing from now on.
   */
  public free(slot: number): void {
    this.batches.withdraw(slot);
    this.holds.release(slot);
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
    this.cull.dispose();
  }

  /** @returns What of a pool's elements are handed out, against what it holds. */
  private static toUse(runs: IStaticRuns): IRendererPoolUse {
    return { capacity: runs.capacity, used: runs.used };
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
    this.batches.withdraw(slot);
    this.holds.release(slot);
    this.pool.write(slot, EStaticSlotKind.NONE, { count: 0, start: 0 }, 0, STATIC_NO_BATCH, STATIC_NO_BATCH);

    if (isCounted) {
      this.fallbacks += 1;
    }

    return false;
  }

  /** Grows the slots once for what the queue brings. */
  private growSlots(): boolean {
    const capacity: number = toGrownCapacity(
      this.pool.count,
      1 + this.toDemand(EStaticPool.SLOTS),
      this.pool.capacity,
      this.buffers.initial(EStaticPool.SLOTS),
      this.buffers.limit(EStaticPool.SLOTS)
    );

    if (capacity <= this.pool.capacity) {
      return false;
    }

    this.buffers.grow(EStaticPool.SLOTS, capacity);

    return true;
  }

  /**
   * @returns A run of a pool, grown once where none fits (`allocateGrowing`), for what the queue brings besides; null
   *   where the device's limit stops it, counted as a fallback.
   */
  private allocateRun(pool: StaticRunPool, count: number): Nullable<number> {
    const start: Nullable<number> = allocateGrowing(
      pool,
      count,
      this.buffers.initial(pool.kind),
      this.buffers.limit(pool.kind),
      (capacity: number) => pool.grow(capacity),
      () => this.toDemand(pool.kind)
    );

    if (start === null) {
      this.fallbacks += 1;
    }

    return start;
  }

  /** What the objects still waiting to draw will take of a pool, besides what they hold already. */
  private toDemand(pool: EStaticPool): number {
    const toTaken: Maybe<(upcoming: IStaticUpcoming) => number> = UPCOMING_DEMAND[pool];
    let demand: number = 0;

    if (toTaken) {
      for (const upcoming of this.toUpcoming()) {
        demand += toTaken(upcoming);
      }
    }

    return demand;
  }
}
