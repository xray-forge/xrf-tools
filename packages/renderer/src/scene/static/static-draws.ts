import { Maybe, Nullable } from "@xrf/types";
import { Material, Matrix4, Object3D, Scene, Sphere } from "three/webgpu";

import { IRendererInstances } from "#/contract/scene/renderer-object";
import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticArenas } from "#/scene/static/static-arenas";
import { StaticBatches } from "#/scene/static/static-batches";
import { StaticCull } from "#/scene/static/static-cull";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticPlaces } from "#/scene/static/static-places";
import { IStaticRange } from "#/scene/static/static-range";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** A run of rows: where it starts, and how many places it tests. */
interface IRowRun {
  start: number;
  count: number;
}

/**
 * Everything drawing static draws: the slots each draw's arguments, sphere and matrix sit in, the places and rows of
 * the instanced ones, the arenas their geometry is copied into, the batches issuing them, one object a material, arena
 * and kind, and the cull on the GPU.
 */
export class StaticDraws {
  /** What culls the static draws on the GPU, which the frame dispatches before drawing them and again between. */
  public readonly cull: StaticCull;
  /** Where the batches' second draws stand, drawn into the G-buffer after the second cull. */
  public readonly late: Scene = new Scene();

  private readonly pool: StaticDrawPool;
  private readonly places: StaticPlaces;
  private readonly arenas: StaticArenas;
  /** The rows each instanced draw's slot tests its places by. */
  private readonly rows: Map<number, IRowRun> = new Map();
  private readonly batches: StaticBatches;

  /**
   * @param buffers - What every static draw reads.
   * @param scene - The scene of the pass drawing static draws, where the batches stand.
   * @param toUpcoming - The geometries objects still waiting to draw will draw statically.
   */
  public constructor(buffers: StaticDrawBuffers, scene: Object3D, toUpcoming: () => Iterable<SceneGeometry>) {
    this.pool = new StaticDrawPool(buffers);
    this.places = new StaticPlaces(buffers);
    this.late.matrixWorldAutoUpdate = false;
    this.cull = new StaticCull(buffers, this.pool, this.places, this.late);
    this.batches = new StaticBatches(this.pool, scene, this.late);
    this.arenas = new StaticArenas(
      (arena: StaticArena) => this.batches.refresh(arena),
      (arena: StaticArena) => this.batches.release(arena),
      toUpcoming
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

  /** Every material a batch draws. */
  public get materials(): Iterable<Material> {
    return this.batches.materials;
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
   * @returns A slot for one static draw, or null where static draws are off or every slot is taken.
   */
  public allocate(): Nullable<number> {
    return this.pool.allocate();
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
    this.batches.put(slot, range.arena, EStaticDrawKind.SINGLE, surface);
  }

  /**
   * @param count - Places an instanced object stands in.
   * @returns Where its places start, or null where there is no room and it has to be drawn plainly.
   */
  public allocatePlaces(count: number): Nullable<number> {
    return this.pool.isEnabled ? this.places.allocatePlaces(count) : null;
  }

  /**
   * @param start - Where an instanced object's places start.
   * @param instances - Its places, as the consumer put them.
   * @param placement - Its own matrix, which places every instance.
   */
  public writePlaces(start: number, instances: IRendererInstances, placement: Matrix4): void {
    this.places.writePlaces(start, instances, placement);
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
   * @returns Whether it is drawn so; not where there is no room for its rows.
   */
  public drawListed(
    slot: number,
    surface: ISurfaceMaterial,
    range: IStaticRange,
    start: number,
    count: number,
    placeStart: number,
    spheres: Float32Array
  ): boolean {
    const places: number = spheres.length / 4;
    let run: Maybe<IRowRun> = this.rows.get(slot);

    if (run && run.count !== places) {
      this.freeRows(slot);
      run = undefined;
    }

    if (!run) {
      const rowStart: Nullable<number> = this.places.allocateRows(places);

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
      count
    );
    this.pool.writeListed(slot, range.indexStart + start, count, range.vertexStart, run.start);
    this.batches.put(slot, range.arena, EStaticDrawKind.LISTED, surface);

    return true;
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
