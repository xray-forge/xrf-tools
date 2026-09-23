import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, BundleGroup, Material, Mesh } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { createSceneMesh } from "#/scene/object/scene-mesh";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { STATIC_DRAW_ARGUMENTS } from "#/uniforms/static-draw-buffers";

/** Bytes one static draw's indirect arguments take. */
const ARGUMENT_BYTES: number = STATIC_DRAW_ARGUMENTS * Uint32Array.BYTES_PER_ELEMENT;

/** What an idle batch's mesh holds instead of the material it last drew, so that one can go. */
const IDLE_MATERIAL: Material = new Material();

/**
 * Every static draw of one material over one arena, as one object issuing an indirect draw a slot: recorded once in a
 * bundle of its own and recorded again only when its draws change. An idle batch keeps its geometry for the next
 * material to draw over the arena, since disposing it would free the arena's buffers every other batch draws.
 */
export class StaticBatch {
  /** What stands for it in the scene of the pass drawing it. */
  public readonly bundle: BundleGroup = new BundleGroup();
  /** The arena it draws. */
  public readonly arena: StaticArena;

  private currentSurface: Nullable<ISurfaceMaterial> = null;
  private geometry: BufferGeometry;
  private mesh: Mesh;
  /** The slots it draws, and each one's position among its draws. */
  private readonly slots: Array<number> = [];
  private readonly positions: Map<number, number> = new Map();
  /** Each draw's offset into the arguments, in the order of its slots. */
  private readonly offsets: Array<number> = [];
  private generation: number;

  /**
   * @param arena - The arena it draws.
   * @param pool - The slots it draws, whose arguments its draws read.
   */
  public constructor(arena: StaticArena, pool: StaticDrawPool) {
    this.arena = arena;
    this.generation = arena.generation;
    this.geometry = this.createGeometry(pool);
    this.mesh = createSceneMesh(this.geometry, null, IDLE_MATERIAL);
    this.bundle.add(this.mesh);
  }

  /** The surface it draws, or null while idle. */
  public get surface(): Nullable<ISurfaceMaterial> {
    return this.currentSurface;
  }

  public get isEmpty(): boolean {
    return this.slots.length === 0;
  }

  /**
   * @param surface - What it draws from now on, or null for a batch going idle.
   */
  public setSurface(surface: Nullable<ISurfaceMaterial>): void {
    this.currentSurface = surface;
    this.mesh.material = surface?.material ?? IDLE_MATERIAL;
    this.bundle.needsUpdate = true;
  }

  /**
   * @param slot - A slot drawn by this batch from now on.
   */
  public add(slot: number): void {
    this.positions.set(slot, this.slots.length);
    this.slots.push(slot);
    this.offsets.push(slot * ARGUMENT_BYTES);
    this.bundle.needsUpdate = true;
  }

  /**
   * @param slot - A slot this batch no longer draws; the last draw takes its place.
   */
  public remove(slot: number): void {
    const position: Maybe<number> = this.positions.get(slot);

    if (position === undefined) {
      return;
    }

    const last: number = this.slots.pop() as number;

    this.offsets.pop();
    this.positions.delete(slot);

    if (last !== slot) {
      this.slots[position] = last;
      this.offsets[position] = last * ARGUMENT_BYTES;
      this.positions.set(last, position);
    }

    this.bundle.needsUpdate = true;
  }

  /** Has its bundle recorded again, for a binding in it that changed: a texture swapped in, or a matrix rewritten. */
  public invalidate(): void {
    this.bundle.needsUpdate = true;
  }

  /**
   * Draws the arena's buffers as they are now, where it grew since: a new mesh over a new geometry, since three keeps
   * what it built for a mesh's first geometry.
   *
   * @param pool - The slots it draws.
   */
  public refresh(pool: StaticDrawPool): void {
    if (this.generation === this.arena.generation) {
      return;
    }

    const material: Material = this.mesh.material as Material;

    this.generation = this.arena.generation;
    this.mesh.removeFromParent();
    this.geometry.dispose();
    this.geometry = this.createGeometry(pool);
    this.mesh = createSceneMesh(this.geometry, null, material);
    this.bundle.add(this.mesh);
    this.bundle.needsUpdate = true;
  }

  /** Frees its geometry, and with it the arena's buffers: only for an arena going, with every batch over it. */
  public dispose(): void {
    this.bundle.removeFromParent();
    this.mesh.removeFromParent();
    this.geometry.dispose();
  }

  private createGeometry(pool: StaticDrawPool): BufferGeometry {
    const geometry: BufferGeometry = this.arena.createGeometry();

    geometry.setIndirect(pool.args, this.offsets);
    // Drawn by its indirect arguments alone; the range only keeps three's count of what a recording drew honest.
    geometry.setDrawRange(0, 0);

    return geometry;
  }
}
