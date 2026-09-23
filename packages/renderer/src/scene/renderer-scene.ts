import { Maybe, Nullable } from "@xrf/types";
import {
  BufferGeometry,
  Material,
  Matrix4,
  Mesh,
  MeshBasicNodeMaterial,
  Scene,
  Skeleton,
  SkinnedMesh,
} from "three/webgpu";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererObject } from "#/contract/scene/renderer-object";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { createRendererBufferGeometry } from "#/scene/renderer-buffer-geometry";
import { RendererSkeletonEntry, RendererSkeletons } from "#/scene/renderer-skeletons";
import { RendererTextures } from "#/scene/renderer-textures";
import { createSurfaceMaterial, ISurfaceMaterial, ISurfaceShadingContext } from "#/scene/surface-material";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";

/** What a slot draws with in the half of an object that does not draw it, or whose surface is missing. */
const HIDDEN: Material = new MeshBasicNodeMaterial({ visible: false });

/** Superseded materials kept compiled, so a toggle back to one draws it at once. */
const MATERIAL_CACHE_LIMIT: number = 64;

/**  One object: a deferred and a forward mesh over one geometry, each drawing only its own slots. */
interface ISceneObject {
  /** The key it was put under, which a released object no longer answers to. */
  key: string;
  object: IRendererObject;
  deferred: Mesh;
  forward: Mesh;
  /** The skeleton both meshes are bound to, or null for rigid meshes. */
  skeleton: Nullable<Skeleton>;
}

/** What an object is about to draw, once every material in it is compiled. */
interface IObjectState {
  geometry: BufferGeometry;
  skeleton: Nullable<Skeleton>;
  deferred: Array<Material>;
  forward: Array<Material>;
}

/**
 * Meshes standing in for objects whose materials are not compiled yet, for the renderer to compile off the frame.
 */
export interface IRendererSceneStaging {
  deferred: Scene;
  forward: Scene;
  /** The materials the staging compiles, to mark ready once it has. */
  materials: ReadonlyArray<Material>;
}

/**
 * Everything a consumer put, as the two scenes the frame draws.
 * A changed surface compiles off the frame: until its material is ready, whatever drew before keeps drawing.
 */
export class RendererScene {
  /**
   * A mesh the object's matrix places directly, never recomposed from a position and rotation.
   *
   * @param skeleton - What it is skinned to, or null for a rigid mesh.
   * @returns The mesh.
   */
  private static createMesh(skeleton: Nullable<Skeleton>): Mesh {
    const mesh: Mesh = skeleton ? new SkinnedMesh() : new Mesh();

    mesh.matrixAutoUpdate = false;

    if (skeleton) {
      // Measured against its bind pose, a skinned mesh would be culled by a motion reaching outside it.
      mesh.frustumCulled = false;
      // Identity: the vertices and the bone transforms are both in model space already.
      (mesh as SkinnedMesh).bind(skeleton, new Matrix4());
    }

    return mesh;
  }

  /** What fills the G-buffer. */
  public readonly deferred: Scene = new Scene();
  /** What is composited after it. */
  public readonly forward: Scene = new Scene();
  public readonly textures: RendererTextures;
  public readonly skeletons: RendererSkeletons;

  private readonly geometries: Map<string, BufferGeometry> = new Map();
  private readonly surfaces: Map<string, ISurfaceMaterial> = new Map();
  /** What each surface was put as, so putting it again unchanged builds nothing. */
  private readonly surfaceDescriptions: Map<string, string> = new Map();
  private readonly objects: Map<string, ISceneObject> = new Map();
  /** What each surface was put as, to build it again when the wireframe setting changes. */
  private readonly surfaceSources: Map<string, IRendererSurface> = new Map();
  /** Materials whose pipelines exist, so drawing them stalls nothing. */
  private readonly ready: WeakSet<Material> = new WeakSet([HIDDEN]);
  /** Objects changed since the scene last settled, drawing what they drew before until every one of them can draw. */
  private readonly pending: Set<ISceneObject> = new Set();
  /** Released objects' meshes, still drawn until the scene settles, so a replacement never leaves a gap. */
  private readonly leaving: Array<Mesh> = [];
  /** Replaced or released geometries, disposed once nothing draws them. */
  private readonly leavingGeometries: Set<BufferGeometry> = new Set();
  /** Textures released while the scene had changes waiting, let go when it settles. */
  private readonly leavingTextures: Set<string> = new Set();
  /** Materials no surface names any more, disposed or cached once nothing draws them. */
  private readonly retired: Set<ISurfaceMaterial> = new Set();
  /** Compiled materials no surface names, by description, for a surface put again as one of them. */
  private readonly cache: Map<string, ISurfaceMaterial> = new Map();
  /** The description each material was built from, which is the cache's key for it. */
  private readonly descriptions: WeakMap<ISurfaceMaterial, string> = new WeakMap();

  private readonly shading: ISurfaceShadingContext;

  private isWireframe: boolean = false;
  /** How deep in `transact` the scene is: nothing settles until the outermost one ends. */
  private depth: number = 0;

  public constructor(shading: ISurfaceShadingContext, onTextureRefused: (key: string, refusal: IDdsRefusal) => void) {
    this.shading = shading;
    this.textures = new RendererTextures(onTextureRefused);
    this.skeletons = new RendererSkeletons((key: string) =>
      this.rebuild((object: IRendererObject) => object.skeleton === key)
    );
  }

  /**
   * @param isWireframe - Whether every surface draws as its triangles' edges.
   */
  public setWireframe(isWireframe: boolean): void {
    if (isWireframe === this.isWireframe) {
      return;
    }

    this.isWireframe = isWireframe;
    this.surfaceSources.forEach((surface: IRendererSurface, key: string) => this.replaceSurface(key, surface));
  }

  /**
   * Makes a run of changes one change: the scene settles only once they are all made.
   *
   * @param change - What to change.
   */
  public transact(change: () => void): void {
    this.depth += 1;

    try {
      change();
    } finally {
      this.depth -= 1;
    }

    this.settle();
  }

  /** Whether any object waits for a material to compile. */
  public get hasPending(): boolean {
    return this.pending.size > 0;
  }

  /**
   * Stands the waiting objects in scenes of their own, as they will draw, for the renderer to compile.
   *
   * @returns The staging, or null when nothing waits.
   */
  public stage(): Nullable<IRendererSceneStaging> {
    if (!this.pending.size) {
      return null;
    }

    const staging: IRendererSceneStaging = { deferred: new Scene(), forward: new Scene(), materials: [] };
    const materials: Set<Material> = new Set();

    for (const entry of this.pending) {
      const state: Nullable<IObjectState> = this.toState(entry);

      if (!state) {
        continue;
      }

      for (const [scene, slots] of [
        [staging.deferred, state.deferred],
        [staging.forward, state.forward],
      ] as const) {
        const mesh: Mesh = RendererScene.createMesh(state.skeleton);

        mesh.geometry = state.geometry;
        mesh.material = slots;
        // Compiled whatever the camera sees: culling would skip what is about to come into view.
        mesh.frustumCulled = false;
        scene.add(mesh);
        slots.filter((material: Material) => !this.ready.has(material)).forEach((it) => materials.add(it));
      }
    }

    return { ...staging, materials: [...materials] };
  }

  /**
   * Marks what a staging compiled as ready, and settles the scene if nothing else is left to compile.
   *
   * @param staging - What was compiled.
   */
  public commit(staging: IRendererSceneStaging): void {
    staging.materials.forEach((material: Material) => this.ready.add(material));
    this.settle();
  }

  public putTexture(key: string, source: TRendererTextureSource): void {
    this.leavingTextures.delete(key);
    this.textures.put(key, source);
  }

  public releaseTexture(key: string): void {
    // Still sampled by whatever keeps drawing until the scene settles, so let go of only then.
    if (this.pending.size || this.depth) {
      this.leavingTextures.add(key);
    } else {
      this.textures.release(key);
    }
  }

  public putGeometry(key: string, geometry: IRendererGeometry): void {
    this.leaveGeometry(key);
    this.geometries.set(key, createRendererBufferGeometry(geometry));
    this.rebuild((object: IRendererObject) => object.geometry === key);
  }

  public releaseGeometry(key: string): void {
    this.leaveGeometry(key);
    this.geometries.delete(key);
    this.rebuild((object: IRendererObject) => object.geometry === key);
  }

  public putSurface(key: string, surface: IRendererSurface): void {
    // The same surface again keeps its material: building one compiles its shader.
    if (this.surfaceDescriptions.get(key) === this.toDescription(surface)) {
      return;
    }

    this.surfaceSources.set(key, surface);
    this.replaceSurface(key, surface);
  }

  public releaseSurface(key: string): void {
    const previous: Maybe<ISurfaceMaterial> = this.surfaces.get(key);

    this.surfaceDescriptions.delete(key);
    this.surfaceSources.delete(key);
    this.surfaces.delete(key);
    this.rebuild((object: IRendererObject) => object.surfaces.includes(key));

    if (previous) {
      this.retired.add(previous);
      this.retire();
    }
  }

  public putObject(key: string, object: IRendererObject): void {
    let entry: Maybe<ISceneObject> = this.objects.get(key);

    if (!entry) {
      entry = {
        deferred: RendererScene.createMesh(null),
        key,
        forward: RendererScene.createMesh(null),
        object,
        skeleton: null,
      };
      this.objects.set(key, entry);
    }

    entry.object = object;
    this.build(entry);
  }

  public releaseObject(key: string): void {
    const entry: Maybe<ISceneObject> = this.objects.get(key);

    if (entry) {
      // Out of the waiting set, or the next settle would draw it again, and drawn on until then, so what replaces
      // it appears in the same frame it goes.
      this.pending.delete(entry);
      this.objects.delete(key);
      this.leaving.push(entry.deferred, entry.forward);
      this.settle();
    }
  }

  public dispose(): void {
    this.objects.forEach((entry: ISceneObject) => {
      entry.deferred.removeFromParent();
      entry.forward.removeFromParent();
    });
    this.objects.clear();
    this.leaving.forEach((mesh: Mesh) => mesh.removeFromParent());
    this.leaving.length = 0;
    this.leavingGeometries.forEach((geometry: BufferGeometry) => geometry.dispose());
    this.leavingGeometries.clear();
    this.leavingTextures.clear();
    this.surfaces.forEach((surface: ISurfaceMaterial) => surface.dispose());
    this.surfaces.clear();
    this.retired.forEach((surface: ISurfaceMaterial) => surface.dispose());
    this.retired.clear();
    this.cache.forEach((surface: ISurfaceMaterial) => surface.dispose());
    this.cache.clear();
    this.pending.clear();
    this.surfaceDescriptions.clear();
    this.surfaceSources.clear();
    this.geometries.forEach((geometry: BufferGeometry) => geometry.dispose());
    this.geometries.clear();
    this.skeletons.dispose();
    this.textures.dispose();
  }

  private rebuild(isAffected: (object: IRendererObject) => boolean): void {
    this.objects.forEach((entry: ISceneObject) => {
      if (isAffected(entry.object)) {
        this.build(entry);
      }
    });
  }

  /** A surface's material for its description now, from the cache when one was compiled for it before. */
  private replaceSurface(key: string, surface: IRendererSurface): void {
    const description: string = this.toDescription(surface);
    const previous: Maybe<ISurfaceMaterial> = this.surfaces.get(key);
    let material: Maybe<ISurfaceMaterial> = this.cache.get(description);

    if (material) {
      this.cache.delete(description);
    } else {
      material = createSurfaceMaterial(surface, this.textures, this.shading);
      material.material.wireframe = this.isWireframe;
      this.descriptions.set(material, description);
    }

    this.surfaceDescriptions.set(key, description);
    this.surfaces.set(key, material);
    this.rebuild((object: IRendererObject) => object.surfaces.includes(key));

    if (previous) {
      this.retired.add(previous);
      this.retire();
    }
  }

  /** What a surface is built from: the surface itself and the wireframe setting, as one comparable string. */
  private toDescription(surface: IRendererSurface): string {
    return `${this.isWireframe ? "wireframe" : "solid"}:${JSON.stringify(surface)}`;
  }

  /** Caches, or disposes past the cache's limit, every retired material nothing draws any more. */
  private retire(): void {
    const drawn: Set<Material> = new Set();
    const meshes: Array<Mesh> = [...this.leaving];

    this.objects.forEach((entry: ISceneObject) => meshes.push(entry.deferred, entry.forward));

    for (const mesh of meshes) {
      (Array.isArray(mesh.material) ? mesh.material : [mesh.material]).forEach((it: Material) => drawn.add(it));
    }

    for (const surface of this.retired) {
      if (drawn.has(surface.material)) {
        continue;
      }

      this.retired.delete(surface);

      const description: Maybe<string> = this.descriptions.get(surface);

      if (description && this.ready.has(surface.material) && !this.cache.has(description)) {
        this.cache.set(description, surface);
      } else {
        surface.dispose();
      }
    }

    while (this.cache.size > MATERIAL_CACHE_LIMIT) {
      const [description, surface] = this.cache.entries().next().value as [string, ISurfaceMaterial];

      this.cache.delete(description);
      surface.dispose();
    }
  }

  /** What an object would draw now, or null for one whose geometry is missing. */
  private toState(entry: ISceneObject): Nullable<IObjectState> {
    const { object } = entry;
    const geometry: Maybe<BufferGeometry> = this.geometries.get(object.geometry);

    if (!geometry) {
      return null;
    }

    // Skinned only when the object names a skeleton that exists and its geometry carries the links to bind with.
    const skeletonEntry: Maybe<RendererSkeletonEntry> = object.skeleton
      ? this.skeletons.get(object.skeleton)
      : undefined;
    const skeleton: Nullable<Skeleton> =
      skeletonEntry && geometry.hasAttribute("skinIndex") ? skeletonEntry.skeleton : null;
    const slots: number = Math.max(
      object.surfaces.length,
      ...geometry.groups.map((group) => (group.materialIndex ?? 0) + 1)
    );
    const surfaces: Array<Maybe<ISurfaceMaterial>> = Array.from({ length: slots }, (_, slot: number) =>
      this.surfaces.get(object.surfaces[slot])
    );

    function toSlots(isDeferred: boolean): Array<Material> {
      return surfaces.map((surface: Maybe<ISurfaceMaterial>) =>
        surface && surface.isDeferred === isDeferred ? surface.material : HIDDEN
      );
    }

    return { deferred: toSlots(true), forward: toSlots(false), geometry, skeleton };
  }

  /** Queues an object's change, and settles the scene if everything changed can draw now. */
  private build(entry: ISceneObject): void {
    // Only an object still held draws: one released while it waited is gone for good.
    if (this.objects.get(entry.key) === entry) {
      this.pending.add(entry);
      this.settle();
    }
  }

  /**
   * Applies every change since the last settle in one go, once every material they name is compiled: what a consumer
   * changed together appears together, and what it released goes in the same frame.
   */
  private settle(): void {
    if (this.depth) {
      return;
    }

    for (const entry of this.pending) {
      const state: Nullable<IObjectState> = this.toState(entry);

      if (state && [...state.deferred, ...state.forward].some((material: Material) => !this.ready.has(material))) {
        return;
      }
    }

    const entries: Array<ISceneObject> = [...this.pending];

    this.pending.clear();
    entries.forEach((entry: ISceneObject) => this.apply(entry));
    this.leaving.forEach((mesh: Mesh) => mesh.removeFromParent());
    this.leaving.length = 0;
    this.leavingGeometries.forEach((geometry: BufferGeometry) => geometry.dispose());
    this.leavingGeometries.clear();
    this.leavingTextures.forEach((key: string) => this.textures.release(key));
    this.leavingTextures.clear();
    this.retire();
  }

  /** A geometry no longer put under its key, disposed once the scene settles and nothing draws it. */
  private leaveGeometry(key: string): void {
    const geometry: Maybe<BufferGeometry> = this.geometries.get(key);

    if (geometry) {
      this.leavingGeometries.add(geometry);
    }
  }

  /** Points both meshes at what the object names now, and shows each only where it has a slot to draw. */
  private apply(entry: ISceneObject): void {
    const state: Nullable<IObjectState> = this.toState(entry);

    if (!state) {
      entry.deferred.removeFromParent();
      entry.forward.removeFromParent();

      return;
    }

    const { object } = entry;

    if (state.skeleton !== entry.skeleton) {
      entry.deferred.removeFromParent();
      entry.forward.removeFromParent();
      entry.deferred = RendererScene.createMesh(state.skeleton);
      entry.forward = RendererScene.createMesh(state.skeleton);
      entry.skeleton = state.skeleton;
    }

    state.geometry.setDrawRange(object.drawRange?.start ?? 0, object.drawRange?.count ?? Infinity);

    for (const [mesh, scene, materials] of [
      [entry.deferred, this.deferred, state.deferred],
      [entry.forward, this.forward, state.forward],
    ] as const) {
      mesh.geometry = state.geometry;
      mesh.material = materials;

      if (object.matrix) {
        mesh.matrix.fromArray(object.matrix);
      } else {
        mesh.matrix.identity();
      }

      mesh.matrixWorldNeedsUpdate = true;

      if (materials.some((material: Material) => material !== HIDDEN)) {
        scene.add(mesh);
      } else {
        mesh.removeFromParent();
      }
    }
  }
}
