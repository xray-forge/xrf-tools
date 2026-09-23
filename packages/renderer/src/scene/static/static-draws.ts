import { Nullable } from "@xrf/types";
import { Material, Matrix4, Object3D, Sphere } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticArenas } from "#/scene/static/static-arenas";
import { StaticBatches } from "#/scene/static/static-batches";
import { StaticCull } from "#/scene/static/static-cull";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { IStaticRange } from "#/scene/static/static-range";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/**
 * Everything drawing static draws: the slots each draw's arguments, sphere and matrix sit in, the arenas their
 * geometry is copied into, the batches issuing them, one object a material and arena, and the cull on the GPU.
 */
export class StaticDraws {
  /** What culls the static draws on the GPU, which the frame dispatches before drawing them. */
  public readonly cull: StaticCull;

  private readonly pool: StaticDrawPool;
  private readonly arenas: StaticArenas;
  private readonly batches: StaticBatches;

  /**
   * @param buffers - What every static draw reads.
   * @param scene - The scene of the pass drawing static draws, where the batches stand.
   */
  public constructor(buffers: StaticDrawBuffers, scene: Object3D) {
    this.pool = new StaticDrawPool(buffers);
    this.cull = new StaticCull(buffers, this.pool);
    this.batches = new StaticBatches(this.pool, scene);
    this.arenas = new StaticArenas(
      (arena: StaticArena) => this.batches.refresh(arena),
      (arena: StaticArena) => this.batches.release(arena)
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
    this.pool.write(slot, range.indexStart + start, count, range.vertexStart, sphere, matrix);
    this.batches.put(slot, range.arena, surface);
  }

  /**
   * @param slot - A slot drawing nothing from now on.
   */
  public free(slot: number): void {
    this.batches.withdraw(slot);
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
}
