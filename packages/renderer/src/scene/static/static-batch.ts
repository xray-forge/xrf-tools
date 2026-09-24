import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, BundleGroup, IndirectStorageBufferAttribute, Material, Mesh, Object3D } from "three/webgpu";

import { createSceneMesh } from "#/scene/object/scene-mesh";
import { StaticArena } from "#/scene/static/static-arena";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { STATIC_DRAW_ARGUMENTS } from "#/uniforms/static-draw-buffers";

/** Bytes one static draw's indirect arguments take. */
const ARGUMENT_BYTES: number = STATIC_DRAW_ARGUMENTS * Uint32Array.BYTES_PER_ELEMENT;

/** What an idle batch's mesh holds instead of the material it last drew, so that one can go. */
const IDLE_MATERIAL: Material = new Material();

/** The arguments one phase of a batch draws by, read again whenever the slots' growth replaced them. */
export type TStaticBatchArguments = () => IndirectStorageBufferAttribute;

/** One phase's draw of a batch: its mesh over the arena, drawn by that phase's arguments. */
interface IBatchPhase {
  toArgs: TStaticBatchArguments;
  geometry: BufferGeometry;
  mesh: Mesh;
}

/**
 * Every static draw of one material and kind over one arena, as one object issuing an indirect draw a slot, once a
 * phase: a surface's batch draws by the first cull's arguments and by the second's, a shadow material's by each
 * cascade's. Its meshes are recorded in the bundles of a chunk of batches, which record again only when a batch in
 * them changes. An idle batch keeps its geometries for the next material to draw over the arena, since disposing them
 * would free the arena's buffers every other batch draws.
 */
export class StaticBatch {
  /** The arena it draws. */
  public readonly arena: StaticArena;
  /** The kind of static draw it issues, which its geometry is marked for. */
  public readonly kind: EStaticDrawKind;

  private readonly phases: ReadonlyArray<IBatchPhase>;
  private currentMaterial: Nullable<Material> = null;
  private currentKeys: ReadonlyArray<string> = [];
  /** The slots it draws, and each one's position among its draws. */
  private readonly slots: Array<number> = [];
  private readonly positions: Map<number, number> = new Map();
  /** Each draw's offset into the arguments of every phase, in the order of its slots. */
  private readonly offsets: Array<number> = [];
  private generation: number;

  /**
   * @param arena - The arena it draws.
   * @param kind - The kind of static draw it issues.
   * @param phases - The arguments each of its phases draws by.
   */
  public constructor(arena: StaticArena, kind: EStaticDrawKind, phases: ReadonlyArray<TStaticBatchArguments>) {
    this.arena = arena;
    this.kind = kind;
    this.generation = arena.generation;
    this.phases = phases.map((toArgs: TStaticBatchArguments) => {
      const geometry: BufferGeometry = this.createGeometry(toArgs());

      return { geometry, mesh: createSceneMesh(geometry, null, IDLE_MATERIAL), toArgs };
    });
  }

  /** Its meshes, a phase each, in the order its phases were given. */
  public get meshes(): ReadonlyArray<Mesh> {
    return this.phases.map((phase: IBatchPhase) => phase.mesh);
  }

  /** The material it draws, or null while idle. */
  public get material(): Nullable<Material> {
    return this.currentMaterial;
  }

  /** The texture keys its material samples, whose swaps it records again for. */
  public get keys(): ReadonlyArray<string> {
    return this.currentKeys;
  }

  public get isEmpty(): boolean {
    return this.slots.length === 0;
  }

  /**
   * @param material - What it draws from now on, or null for a batch going idle.
   * @param keys - The texture keys that material samples.
   */
  public setMaterial(material: Nullable<Material>, keys: ReadonlyArray<string> = []): void {
    this.currentMaterial = material;
    this.currentKeys = keys;
    this.phases.forEach((phase: IBatchPhase) => (phase.mesh.material = material ?? IDLE_MATERIAL));
    this.invalidate();
  }

  /**
   * @param slot - A slot drawn by this batch from now on.
   */
  public add(slot: number): void {
    this.positions.set(slot, this.slots.length);
    this.slots.push(slot);
    this.offsets.push(slot * ARGUMENT_BYTES);
    this.invalidate();
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

    this.invalidate();
  }

  /** Has its bundles recorded again, for a binding in them that changed: a texture swapped in, or a matrix rewritten. */
  public invalidate(): void {
    for (const { mesh } of this.phases) {
      if (mesh.parent instanceof BundleGroup) {
        mesh.parent.needsUpdate = true;
      }
    }
  }

  /**
   * Draws the arena's buffers and the slots' arguments as they are now, where either grew since: new meshes over new
   * geometries, since three keeps what it built for a mesh's first geometry.
   */
  public refresh(): void {
    if (this.generation === this.arena.generation) {
      return;
    }

    this.generation = this.arena.generation;

    for (const phase of this.phases) {
      const material: Material = phase.mesh.material as Material;
      const parent: Nullable<Object3D> = phase.mesh.parent;

      phase.mesh.removeFromParent();
      phase.geometry.dispose();
      phase.geometry = this.createGeometry(phase.toArgs());
      phase.mesh = createSceneMesh(phase.geometry, null, material);
      parent?.add(phase.mesh);
    }

    this.invalidate();
  }

  /** Frees its geometries, and with them the arena's buffers: only for an arena going, with every batch over it. */
  public dispose(): void {
    for (const phase of this.phases) {
      phase.mesh.removeFromParent();
      phase.geometry.dispose();
    }
  }

  private createGeometry(args: IndirectStorageBufferAttribute): BufferGeometry {
    const geometry: BufferGeometry = this.arena.createGeometry(this.kind);

    geometry.setIndirect(args, this.offsets);
    // Drawn by its indirect arguments alone; the range only keeps three's count of what a recording drew honest.
    geometry.setDrawRange(0, 0);

    return geometry;
  }
}
