import { Maybe, Nullable } from "@xrf/types";
import {
  BufferGeometry,
  BundleGroup,
  IndirectStorageBufferAttribute,
  LineSegments,
  Material,
  Mesh,
  Object3D,
} from "three/webgpu";

import { disposeObject } from "#/internals/object-disposal";
import { createSceneLines, createSceneMesh } from "#/scene/object/scene-mesh";
import { StaticArena } from "#/scene/static/static-arena";
import { IStaticRegion } from "#/scene/static/static-region";
import { EStaticListSpace, STATIC_BATCH_ARGUMENT_BYTES } from "#/uniforms/static-draw-buffers";

/** What an idle batch's mesh holds instead of the material it last drew, so that one can go. */
const IDLE_MATERIAL: Material = new Material();

/** The arguments one phase of a batch draws by, read again whenever the batches' growth replaced them. */
export type TStaticBatchArguments = () => IndirectStorageBufferAttribute;

/** What a batch draws a phase with: a mesh of its clusters' triangles, or line segments of their edges. */
export type TStaticBatchMesh = Mesh | LineSegments;

/** One phase's draw of a batch: its mesh, drawn by that phase's arguments at the batch's offset. */
interface IBatchPhase {
  toArgs: TStaticBatchArguments;
  geometry: BufferGeometry;
  mesh: TStaticBatchMesh;
}

/**
 * Every static draw of one material over one arena, as one object a phase issuing one indirect draw: of every cluster
 * the phase's view kept of them, a cluster an instance, read from the batch's region of the view's list. A surface's
 * batch draws by the camera's two views, a shadow material's by each shadow view's. Its meshes are recorded in the
 * bundles of a chunk of batches, which record again only when a batch in them changes.
 */
export class StaticBatch {
  /** The arena it draws. */
  public readonly arena: StaticArena;
  /** Where its draw sits in every view's arguments, and its region's entry in the regions. */
  public readonly id: number;
  /** The list space its region is in. */
  public readonly space: EStaticListSpace;
  /** Where its region lies in its list space, which it is given as the entries it may list outgrow it. */
  public region: Nullable<IStaticRegion> = null;
  /** Whether its material is an impostor's, which draws its own wireframe. */
  public isImpostor: boolean = false;

  private readonly phases: Array<IBatchPhase>;
  /** Its draws of its clusters' edges, while a wireframe draws, by the wireframe's arguments at the same offset. */
  private wires: Nullable<Array<IBatchPhase>> = null;
  private currentMaterial: Nullable<Material> = null;
  private currentKeys: ReadonlyArray<string> = [];
  /** Entries each slot it draws may list at once: its clusters, times its rows for an instanced one. */
  private readonly entries: Map<number, number> = new Map();
  private currentDemand: number = 0;

  /**
   * @param arena - The arena it draws.
   * @param id - Where its draw sits in every view's arguments.
   * @param space - The list space its region is in.
   * @param phases - The arguments each of its phases draws by.
   */
  public constructor(
    arena: StaticArena,
    id: number,
    space: EStaticListSpace,
    phases: ReadonlyArray<TStaticBatchArguments>
  ) {
    this.arena = arena;
    this.id = id;
    this.space = space;
    this.phases = phases.map((toArgs: TStaticBatchArguments) => this.createPhase(toArgs, IDLE_MATERIAL, false));
  }

  /** Its meshes, a phase each, in the order its phases were given. */
  public get meshes(): ReadonlyArray<TStaticBatchMesh> {
    return this.phases.map((phase: IBatchPhase) => phase.mesh);
  }

  /** Its wireframe's meshes, a phase each, or none before a wireframe first drew it. */
  public get wireMeshes(): ReadonlyArray<TStaticBatchMesh> {
    return this.wires?.map((phase: IBatchPhase) => phase.mesh) ?? [];
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
    return this.entries.size === 0;
  }

  /** Entries its slots may list at once, which its region has to hold. */
  public get demand(): number {
    return this.currentDemand;
  }

  /**
   * @param material - What it draws from now on, or null for a batch going idle.
   * @param keys - The texture keys that material samples.
   * @param isImpostor - Whether it is an impostor's.
   */
  public setMaterial(
    material: Nullable<Material>,
    keys: ReadonlyArray<string> = [],
    isImpostor: boolean = false
  ): void {
    this.currentMaterial = material;
    this.currentKeys = keys;
    this.isImpostor = isImpostor;
    this.phases.forEach((phase: IBatchPhase) => (phase.mesh.material = material ?? IDLE_MATERIAL));
    this.invalidate();
  }

  /**
   * Draws its clusters' edges beside its triangles, each phase by the wireframe's arguments, with the material given.
   *
   * @param wires - The arguments of the wireframe's phases.
   * @param material - What draws the edges.
   */
  public setWire(wires: ReadonlyArray<TStaticBatchArguments>, material: Material): void {
    if (!this.wires) {
      this.wires = wires.map((toArgs: TStaticBatchArguments) => this.createPhase(toArgs, material, true));
    }

    this.wires.forEach((phase: IBatchPhase) => (phase.mesh.material = material));
    this.invalidate();
  }

  /**
   * @param slot - A slot drawn by this batch from now on, or again with what it may list.
   * @param entries - Entries it may list at once.
   */
  public put(slot: number, entries: number): void {
    this.currentDemand += entries - (this.entries.get(slot) ?? 0);
    this.entries.set(slot, entries);
  }

  /**
   * @param slot - A slot this batch no longer draws.
   */
  public remove(slot: number): void {
    const entries: Maybe<number> = this.entries.get(slot);

    if (entries !== undefined) {
      this.currentDemand -= entries;
      this.entries.delete(slot);
    }
  }

  /** Has its bundles recorded again, for a binding in them that changed. */
  public invalidate(): void {
    for (const { mesh } of [...this.phases, ...(this.wires ?? [])]) {
      if (mesh.parent instanceof BundleGroup) {
        mesh.parent.needsUpdate = true;
      }
    }
  }

  /** Draws by the arguments as they are now, where the batches' growth replaced them. */
  public refresh(): void {
    for (const phases of [this.phases, this.wires ?? []]) {
      phases.forEach((phase: IBatchPhase, index: number) => {
        if (phase.geometry.indirect === phase.toArgs()) {
          return;
        }

        const parent: Nullable<Object3D> = phase.mesh.parent;
        const replaced: IBatchPhase = this.createPhase(
          phase.toArgs,
          phase.mesh.material as Material,
          phases === this.wires
        );

        StaticBatch.disposePhase(phase);
        parent?.add(replaced.mesh);
        phases[index] = replaced;
      });
    }

    this.invalidate();
  }

  /** Lets three forget its meshes and geometries: its arena's data is the arena's and stays. */
  public dispose(): void {
    [...this.phases, ...(this.wires ?? [])].forEach(StaticBatch.disposePhase);
  }

  private static disposePhase(phase: IBatchPhase): void {
    phase.mesh.removeFromParent();
    disposeObject(phase.mesh);
    phase.geometry.dispose();
  }

  private createPhase(toArgs: TStaticBatchArguments, material: Material, isWire: boolean): IBatchPhase {
    const geometry: BufferGeometry = this.arena.createGeometry();

    geometry.setIndirect(toArgs(), this.id * STATIC_BATCH_ARGUMENT_BYTES);
    // Drawn by its indirect arguments alone; the range only keeps three's count of what a recording drew honest.
    geometry.setDrawRange(0, 0);

    return {
      geometry,
      mesh: isWire ? createSceneLines(geometry, material) : createSceneMesh(geometry, null, material),
      toArgs,
    };
  }
}
