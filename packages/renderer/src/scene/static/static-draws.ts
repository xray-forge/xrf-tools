import { Maybe, Nullable } from "@xrf/types";
import { Box3, Material, Matrix4, Object3D, Scene, Sphere, Vector3, Vector4 } from "three/webgpu";

import { IRendererPoolUse, IRendererStaticDrawReport } from "#/contract/renderer-report";
import { IRendererInstances } from "#/contract/scene/renderer-object";
import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticArenas } from "#/scene/static/static-arenas";
import { StaticBatches } from "#/scene/static/static-batches";
import { StaticCull } from "#/scene/static/static-cull";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { toGrownCapacity } from "#/scene/static/static-growth";
import { StaticLods } from "#/scene/static/static-lods";
import { StaticPlaces } from "#/scene/static/static-places";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { IStaticUpcoming } from "#/scene/static/static-upcoming";
import { EStaticPool, STATIC_NO_BAND, STATIC_SHADOW_VIEWS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** A run of rows: where it starts, and how many places it tests. */
interface IRowRun {
  start: number;
  count: number;
}

/** What the objects still waiting to draw will take of each pool. */
type TStaticDemand = Record<EStaticPool.SLOTS | EStaticPool.PLACES | EStaticPool.ROWS, number>;

/**
 * Everything drawing static draws: the slots each draw's arguments, sphere and matrix sit in, the places and rows of
 * the instanced ones, the arenas their geometry is copied into, the batches issuing them, one object a material, arena
 * and kind, and the cull on the GPU. A pool that runs out grows, once for everything the queue is known to bring; a
 * draw is drawn plainly only where the device's limit stops it.
 */
/** A corner of a place's sphere, reused. */
const PLACE_CORNER: Vector3 = new Vector3();

export class StaticDraws implements IStaticShadowCasters {
  /** What culls the static draws on the GPU, which the frame dispatches before drawing them and again between. */
  public readonly cull: StaticCull;
  /** Where the batches' second draws stand, drawn into the G-buffer after the second cull. */
  public readonly late: Scene = new Scene();
  /** What each shadow view draws: every casting batch, by the view's arguments. */
  public readonly shadowScenes: ReadonlyArray<Scene> = Array.from({ length: STATIC_SHADOW_VIEWS }, () => new Scene());
  /** What every cascade draws besides the batches: a twin of each part drawn plainly that casts. */
  public readonly plainCasters: Scene = new Scene();

  private readonly buffers: StaticDrawBuffers;
  private readonly pool: StaticDrawPool;
  private readonly places: StaticPlaces;
  /** The impostors of clumps of trees, which the LOD cull decides between a clump and its impostor by. */
  public readonly lods: StaticLods;
  private readonly arenas: StaticArenas;
  /** The rows each instanced draw's slot tests its places by. */
  private readonly rows: Map<number, IRowRun> = new Map();
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
    this.lods = new StaticLods(buffers);
    this.late.matrixWorldAutoUpdate = false;
    this.cull = new StaticCull(buffers, this.pool, this.places, this.lods, this.late);
    this.batches = new StaticBatches(this.pool, scene, this.late, this.shadowScenes);
    this.arenas = new StaticArenas(
      (arena: StaticArena) => this.batches.refresh(arena),
      (arena: StaticArena) => this.batches.release(arena),
      () => [...toUpcoming()].map((upcoming: IStaticUpcoming) => upcoming.geometry),
      () => buffers.capacity(EStaticPool.SLOTS)
    );
  }

  /**
   * Whether static draws are drawn at all: only on a device drawing an indirect draw's first instance, which is how a
   * static draw finds its slot.
   */
  public get isEnabled(): boolean {
    return this.pool.isEnabled;
  }

  public set isEnabled(isEnabled: boolean) {
    this.pool.isEnabled = isEnabled;
  }

  /**
   * @param material - What every static draw's edges draw with, over its arena's line index; null to draw triangles.
   */
  public setWireframe(material: Nullable<Material>): void {
    this.batches.setWireframe(material);
    this.cull.setWireframe(material !== null);
  }

  /** Every material a batch draws. */
  /** Bumped whenever what any batch draws changed, so a shadow map drawn before is drawn again. */
  public get shadowVersion(): number {
    return this.batches.version;
  }

  /**
   * @param view - A cascade.
   * @param planes - Its box's planes.
   */
  public showShadowCells(view: number, planes: ReadonlyArray<Vector4>): void {
    this.batches.showShadowCells(view, planes);
  }

  public get materials(): Iterable<Material> {
    return this.batches.materials;
  }

  /** How full each pool is, how many draws fell back to the plain path, and what the last cull found occluded. */
  public get report(): IRendererStaticDrawReport {
    const slots: IRendererPoolUse = { capacity: this.pool.capacity, used: this.pool.count };
    const { occludedDraws, occludedInstances, occludedTriangles } = this.cull.kept;

    return {
      fallbacks: this.fallbacks,
      occluded: { draws: occludedDraws, instances: occludedInstances, triangles: occludedTriangles },
      lods: this.lods.use,
      places: this.places.placeUse,
      rows: this.places.rowUse,
      slots,
    };
  }

  /**
   * @param geometry - A geometry.
   * @returns The arena its layout sits in, whose prototype its static draws' materials compile against.
   */
  public toArena(geometry: SceneGeometry): StaticArena {
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
   * @param slot - The slot drawing.
   * @param surface - What draws it.
   * @param range - Where its geometry sits.
   * @param start - Its first index, within its geometry's.
   * @param count - Indices it draws; none draws nothing.
   * @param sphere - What it spans in renderer space.
   * @param matrix - Where it stands.
   */
  public draw(
    slot: number,
    surface: ISurfaceMaterial,
    range: IStaticRange,
    start: number,
    count: number,
    sphere: Sphere,
    matrix: Matrix4
  ): void {
    this.freeRows(slot);
    this.pool.write(slot, range.indexStart + start, count, range.vertexStart, sphere, matrix);
    this.batches.put(slot, range.arena, EStaticDrawKind.SINGLE, surface, sphere.getBoundingBox(new Box3()));
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
   * An instanced static draw: its section drawn once for every place the instance cull keeps.
   *
   * @param slot - The slot drawing.
   * @param surface - What draws it.
   * @param range - Where its geometry sits.
   * @param start - Its first index, within its geometry's.
   * @param count - Indices it draws; none draws nothing.
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
    start: number,
    count: number,
    placeStart: number,
    spheres: Float32Array,
    lods: Nullable<Uint32Array> = null,
    band: number = STATIC_NO_BAND
  ): boolean {
    const places: number = spheres.length / 4;
    let run: Maybe<IRowRun> = this.rows.get(slot);

    if (run && run.count !== places) {
      this.freeRows(slot);
      run = undefined;
    }

    if (!run) {
      const rowStart: Nullable<number> = this.allocateRun(EStaticPool.ROWS, places);

      if (rowStart === null) {
        return false;
      }

      run = { count: places, start: rowStart };
      this.rows.set(slot, run);
    }

    this.places.writeRows(
      run.start,
      count ? spheres : new Float32Array(spheres.length).fill(-1),
      placeStart,
      slot,
      count,
      lods,
      band
    );
    this.pool.writeListed(slot, range.indexStart + start, count, range.vertexStart, run.start);
    this.batches.put(slot, range.arena, EStaticDrawKind.LISTED, surface, StaticDraws.toPlacesBox(spheres));

    return true;
  }

  /**
   * @param spheres - Each place's sphere, four floats each, a negative radius for none.
   * @returns The box every place spans, which the draw's cell is taken from.
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
   * @param slot - A slot drawing nothing from now on.
   */
  public free(slot: number): void {
    this.batches.withdraw(slot);
    this.freeRows(slot);
    this.pool.release(slot);
  }

  /**
   * @param key - A texture key whose samplers now sample another texture.
   */
  public invalidate(key: string): void {
    this.batches.invalidate(key);
  }

  /** Grows the slots once for what the queue brings, and every arena's slot attribute with them. */
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
    // A new slot attribute in every arena, so every batch is made again over it and the new arguments.
    this.arenas.growSlots(capacity);

    return true;
  }

  /**
   * @returns A run of places or rows, grown until it fits: first for what the queue brings, then past the whole run,
   *   where freed room lies in runs too short for it. Null where the device's limit stops it.
   */
  private allocateRun(pool: EStaticPool.PLACES | EStaticPool.ROWS | EStaticPool.LODS, count: number): Nullable<number> {
    const allocate = (): Nullable<number> => {
      switch (pool) {
        case EStaticPool.PLACES:
          return this.places.allocatePlaces(count);
        case EStaticPool.ROWS:
          return this.places.allocateRows(count);
        case EStaticPool.LODS:
          return this.lods.allocate(count);
      }
    };
    const limit: number = this.buffers.limit(pool);
    let start: Nullable<number> = allocate();
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

      if (pool === EStaticPool.LODS) {
        this.lods.grow(capacity);
      } else {
        this.places.grow(pool, capacity);
      }

      if (pool === EStaticPool.ROWS) {
        // The second cull lists its places a row capacity on, which moved.
        this.pool.relist();
      }

      // Every material reading the places or the list binds the new buffer once its batch records.
      this.batches.invalidateAll();
      isFirst = false;
      start = allocate();
    }

    return start;
  }

  private toUse(pool: EStaticPool.PLACES | EStaticPool.ROWS | EStaticPool.LODS): IRendererPoolUse {
    switch (pool) {
      case EStaticPool.PLACES:
        return this.places.placeUse;
      case EStaticPool.ROWS:
        return this.places.rowUse;
      case EStaticPool.LODS:
        return this.lods.use;
    }
  }

  /** What the objects still waiting to draw will take of each pool, besides what they hold already. */
  private toDemand(): TStaticDemand {
    const demand: TStaticDemand = { [EStaticPool.PLACES]: 0, [EStaticPool.ROWS]: 0, [EStaticPool.SLOTS]: 0 };

    for (const { sections, places } of this.toUpcoming()) {
      demand[EStaticPool.SLOTS] += sections;
      demand[EStaticPool.PLACES] += places;
      demand[EStaticPool.ROWS] += places * sections;
    }

    return demand;
  }

  private freeRows(slot: number): void {
    const run: Maybe<IRowRun> = this.rows.get(slot);

    if (run) {
      this.places.freeRows(run.start, run.count);
      this.rows.delete(slot);
    }
  }

  public dispose(): void {
    this.batches.dispose();
    this.arenas.dispose();
    this.cull.dispose();
  }
}
