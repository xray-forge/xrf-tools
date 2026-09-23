import { Maybe, Nullable } from "@xrf/types";
import {
  BufferGeometry,
  InstancedBufferAttribute,
  InstancedMesh,
  Material,
  Matrix4,
  Mesh,
  MeshBasicNodeMaterial,
  Scene,
  Skeleton,
  SkinnedMesh,
} from "three/webgpu";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererInstances, IRendererObject } from "#/contract/scene/renderer-object";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { createRendererBufferGeometry } from "#/scene/renderer-buffer-geometry";
import { RendererSkeletonEntry, RendererSkeletons } from "#/scene/renderer-skeletons";
import { RendererTextures } from "#/scene/renderer-textures";
import {
  createSurfaceMaterial,
  ESurfacePass,
  INSTANCE_HEMI_ATTRIBUTE,
  ISurfaceMaterial,
  ISurfaceShadingContext,
} from "#/scene/surface-material";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";

/** What a slot draws with in a pass that does not draw it, or whose surface is missing. */
const HIDDEN: Material = new MeshBasicNodeMaterial({ visible: false });

/** Superseded materials kept compiled, so a toggle back to one draws it at once. */
const MATERIAL_CACHE_LIMIT: number = 64;

/** Floats one instance's transform takes. */
const FLOATS_PER_INSTANCE: number = 16;

/** Every pass an object draws in, in frame order. */
const PASSES: ReadonlyArray<ESurfacePass> = [ESurfacePass.DEFERRED, ESurfacePass.WALLMARK, ESurfacePass.FORWARD];

/** One mesh per pass, each drawing only its own slots. */
type TPassMeshes = Record<ESurfacePass, Mesh>;

/** One scene per pass. */
export type TPassScenes = Record<ESurfacePass, Scene>;

/** What an object stands in many places with: its transforms, uploaded once for every pass drawing it. */
interface IObjectInstances {
  source: IRendererInstances;
  matrices: InstancedBufferAttribute;
}

/** One object: a mesh per pass over one geometry. */
interface ISceneObject {
  /** The key it was put under, which a released object no longer answers to. */
  key: string;
  object: IRendererObject;
  meshes: TPassMeshes;
  /** The skeleton its meshes are bound to, or null for rigid meshes. */
  skeleton: Nullable<Skeleton>;
  /** The places its meshes stand, or null for a mesh its matrix places once. */
  instances: Nullable<IObjectInstances>;
  /** The change it waits in, or null while it draws as it was last put. */
  change: Nullable<ISceneChange>;
}

/** What an object is about to draw, once every material in it is compiled for its layout. */
interface IObjectState {
  geometry: BufferGeometry;
  skeleton: Nullable<Skeleton>;
  instances: Nullable<IObjectInstances>;
  slots: Record<ESurfacePass, Array<Material>>;
  /** The vertex layout the materials compile against. */
  layout: string;
}

/**
 * What a consumer changed together: applied together, once every material it names is compiled, after every change
 * made before it.
 */
interface ISceneChange {
  objects: Set<ISceneObject>;
  /** Released objects' meshes, drawn until the change applies, so a replacement never leaves a gap. */
  leaving: Array<Mesh>;
  /** Replaced or released geometries, disposed once nothing draws them. */
  geometries: Set<BufferGeometry>;
  /** Textures released, let go of once whatever sampled them stops drawing. */
  textures: Set<string>;
}

/**
 * Meshes standing in for objects whose materials are not compiled yet, for the renderer to compile off the frame.
 */
export interface IRendererSceneStaging {
  scenes: TPassScenes;
  /** The materials the staging compiles, each against the layout it compiles for. */
  materials: ReadonlyArray<readonly [Material, string]>;
}

/**
 * Everything a consumer put, as the scenes the frame draws.
 * A change compiles off the frame: until its materials are ready, whatever drew before keeps drawing.
 */
export class RendererScene {
  /**
   * A mesh the object's matrix places directly, never recomposed from a position and rotation.
   *
   * @param skeleton - What it is skinned to, or null for a rigid mesh.
   * @param instances - Where it stands, or null for a mesh standing once.
   * @returns The mesh.
   */
  private static createMesh(skeleton: Nullable<Skeleton>, instances: Nullable<IObjectInstances>): Mesh {
    let mesh: Mesh;

    if (instances) {
      const instanced: InstancedMesh = new InstancedMesh(undefined, undefined, instances.matrices.count);

      instanced.instanceMatrix = instances.matrices;
      mesh = instanced;
    } else {
      mesh = skeleton ? new SkinnedMesh() : new Mesh();
    }

    mesh.matrixAutoUpdate = false;

    if (mesh instanceof SkinnedMesh && skeleton) {
      // Measured against its bind pose, a skinned mesh would be culled by a motion reaching outside it.
      mesh.frustumCulled = false;
      // Identity: the vertices and the bone transforms are both in model space already.
      mesh.bind(skeleton, new Matrix4());
    }

    return mesh;
  }

  private static createMeshes(skeleton: Nullable<Skeleton>, instances: Nullable<IObjectInstances>): TPassMeshes {
    return {
      [ESurfacePass.DEFERRED]: RendererScene.createMesh(skeleton, instances),
      [ESurfacePass.FORWARD]: RendererScene.createMesh(skeleton, instances),
      [ESurfacePass.WALLMARK]: RendererScene.createMesh(skeleton, instances),
    };
  }

  private static createScenes(): TPassScenes {
    return {
      [ESurfacePass.DEFERRED]: new Scene(),
      [ESurfacePass.FORWARD]: new Scene(),
      [ESurfacePass.WALLMARK]: new Scene(),
    };
  }

  /** The vertex layout a mesh compiles against, which three builds a shader per. */
  private static toLayout(geometry: BufferGeometry, skeleton: Nullable<Skeleton>, isInstanced: boolean): string {
    return `${isInstanced ? "instanced" : ""}${skeleton ? "skinned" : ""}:${Object.keys(geometry.attributes).sort()}`;
  }

  /** What each pass draws. */
  public readonly scenes: TPassScenes = RendererScene.createScenes();
  public readonly textures: RendererTextures;
  public readonly skeletons: RendererSkeletons;

  private readonly geometries: Map<string, BufferGeometry> = new Map();
  private readonly surfaces: Map<string, ISurfaceMaterial> = new Map();
  /** What each surface was put as, so putting it again unchanged builds nothing. */
  private readonly surfaceDescriptions: Map<string, string> = new Map();
  /** What each surface was put as, to build it again when the wireframe setting changes. */
  private readonly surfaceSources: Map<string, IRendererSurface> = new Map();
  private readonly objects: Map<string, ISceneObject> = new Map();
  /** The objects each surface key is drawn by, so a put touches them and nothing else. */
  private readonly surfaceUsers: Map<string, Set<ISceneObject>> = new Map();
  /** The objects each geometry key is drawn by. */
  private readonly geometryUsers: Map<string, Set<ISceneObject>> = new Map();
  /** The layouts each material's pipelines exist for, so drawing it in one stalls nothing. */
  private readonly ready: WeakMap<Material, Set<string>> = new WeakMap();
  /** Changes not applied yet, oldest first. */
  private readonly changes: Array<ISceneChange> = [];
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
  /** The change the running transaction adds to, made on its first use. */
  private open: Nullable<ISceneChange> = null;

  public constructor(shading: ISurfaceShadingContext, onTextureRefused: (key: string, refusal: IDdsRefusal) => void) {
    this.shading = shading;
    this.textures = new RendererTextures(onTextureRefused);
    this.skeletons = new RendererSkeletons((key: string) =>
      this.transact(() =>
        this.objects.forEach((entry: ISceneObject) => {
          if (entry.object.skeleton === key) {
            this.build(entry);
          }
        })
      )
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
    this.transact(() =>
      this.surfaceSources.forEach((surface: IRendererSurface, key: string) => this.replaceSurface(key, surface))
    );
  }

  /**
   * Makes a run of changes one change: it applies only once all of it can draw.
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

    if (!this.depth) {
      this.open = null;
      this.settle();
    }
  }

  /** Whether any object waits for a material to compile. */
  public get hasPending(): boolean {
    return this.changes.some((change: ISceneChange) => change.objects.size > 0);
  }

  /**
   * Stands the waiting objects in scenes of their own, as they will draw, for the renderer to compile.
   *
   * @returns The staging, or null when nothing waits.
   */
  public stage(): Nullable<IRendererSceneStaging> {
    if (!this.hasPending) {
      return null;
    }

    const scenes: TPassScenes = RendererScene.createScenes();
    const materials: Map<Material, Set<string>> = new Map();

    for (const change of this.changes) {
      for (const entry of change.objects) {
        const state: Nullable<IObjectState> = this.toState(entry);

        if (!state) {
          continue;
        }

        for (const pass of PASSES) {
          const slots: Array<Material> = state.slots[pass];

          if (slots.every((material: Material) => material === HIDDEN)) {
            continue;
          }

          const mesh: Mesh = RendererScene.createMesh(state.skeleton, state.instances);

          mesh.geometry = state.geometry;
          mesh.material = slots;
          // Compiled whatever the camera sees: culling would skip what is about to come into view.
          mesh.frustumCulled = false;
          scenes[pass].add(mesh);

          for (const material of slots) {
            if (!this.isReady(material, state.layout)) {
              materials.set(material, (materials.get(material) ?? new Set()).add(state.layout));
            }
          }
        }
      }
    }

    return {
      materials: [...materials].flatMap(([material, layouts]) => [...layouts].map((it) => [material, it] as const)),
      scenes,
    };
  }

  /**
   * Marks what a staging compiled as ready, and applies every change that can draw now.
   *
   * @param staging - What was compiled.
   */
  public commit(staging: IRendererSceneStaging): void {
    for (const [material, layout] of staging.materials) {
      this.ready.set(material, (this.ready.get(material) ?? new Set()).add(layout));
    }

    this.transact(() => {});
  }

  public putTexture(key: string, source: TRendererTextureSource): void {
    this.transact(() => {
      // Put again before an earlier release let go of it: the release is superseded.
      this.changes.forEach((change: ISceneChange) => change.textures.delete(key));
      this.textures.put(key, source);
    });
  }

  public releaseTexture(key: string): void {
    // Still sampled by whatever draws until the changes before this one apply, so let go of only then.
    this.transact(() => this.current.textures.add(key));
  }

  public putGeometry(key: string, geometry: IRendererGeometry): void {
    this.transact(() => {
      this.leaveGeometry(key);
      this.geometries.set(key, createRendererBufferGeometry(geometry));
      this.buildUsers(this.geometryUsers.get(key));
    });
  }

  public releaseGeometry(key: string): void {
    this.transact(() => {
      this.leaveGeometry(key);
      this.geometries.delete(key);
      this.buildUsers(this.geometryUsers.get(key));
    });
  }

  public putSurface(key: string, surface: IRendererSurface): void {
    // The same surface again keeps its material: building one compiles its shader.
    if (this.surfaceDescriptions.get(key) === this.toDescription(surface)) {
      return;
    }

    this.surfaceSources.set(key, surface);
    this.transact(() => this.replaceSurface(key, surface));
  }

  public releaseSurface(key: string): void {
    this.transact(() => {
      const previous: Maybe<ISurfaceMaterial> = this.surfaces.get(key);

      this.surfaceDescriptions.delete(key);
      this.surfaceSources.delete(key);
      this.surfaces.delete(key);
      this.buildUsers(this.surfaceUsers.get(key));

      if (previous) {
        this.retired.add(previous);
      }
    });
  }

  public putObject(key: string, object: IRendererObject): void {
    this.transact(() => {
      let entry: Maybe<ISceneObject> = this.objects.get(key);

      if (entry) {
        this.unindex(entry);
      } else {
        entry = {
          change: null,
          instances: null,
          key,
          meshes: RendererScene.createMeshes(null, null),
          object,
          skeleton: null,
        };
        this.objects.set(key, entry);
      }

      entry.object = object;
      this.index(entry);
      this.build(entry);
    });
  }

  public releaseObject(key: string): void {
    const entry: Maybe<ISceneObject> = this.objects.get(key);

    if (!entry) {
      return;
    }

    this.transact(() => {
      const into: ISceneChange = this.current;

      // Whatever it waited in goes with its release, or a geometry that change let go of would be disposed while the
      // meshes leaving here still draw it.
      if (entry.change && entry.change !== into) {
        this.merge(entry.change, into);
      }

      into.objects.delete(entry);
      entry.change = null;
      this.unindex(entry);
      this.objects.delete(key);
      into.leaving.push(...PASSES.map((pass: ESurfacePass) => entry.meshes[pass]));
    });
  }

  public dispose(): void {
    this.objects.forEach((entry: ISceneObject) => PASSES.forEach((pass) => entry.meshes[pass].removeFromParent()));
    this.objects.clear();

    for (const change of this.changes) {
      change.leaving.forEach((mesh: Mesh) => mesh.removeFromParent());
      change.geometries.forEach((geometry: BufferGeometry) => geometry.dispose());
    }

    this.changes.length = 0;
    this.open = null;
    this.surfaces.forEach((surface: ISurfaceMaterial) => surface.dispose());
    this.surfaces.clear();
    this.retired.forEach((surface: ISurfaceMaterial) => surface.dispose());
    this.retired.clear();
    this.cache.forEach((surface: ISurfaceMaterial) => surface.dispose());
    this.cache.clear();
    this.surfaceDescriptions.clear();
    this.surfaceSources.clear();
    this.surfaceUsers.clear();
    this.geometryUsers.clear();
    this.geometries.forEach((geometry: BufferGeometry) => geometry.dispose());
    this.geometries.clear();
    this.skeletons.dispose();
    this.textures.dispose();
  }

  /** The change the running transaction adds to. */
  private get current(): ISceneChange {
    if (!this.open) {
      this.open = { geometries: new Set(), leaving: [], objects: new Set(), textures: new Set() };
      this.changes.push(this.open);
    }

    return this.open;
  }

  private index(entry: ISceneObject): void {
    RendererScene.addUser(this.geometryUsers, entry.object.geometry, entry);
    entry.object.surfaces.forEach((surface: string) => RendererScene.addUser(this.surfaceUsers, surface, entry));
  }

  private unindex(entry: ISceneObject): void {
    RendererScene.removeUser(this.geometryUsers, entry.object.geometry, entry);
    entry.object.surfaces.forEach((surface: string) => RendererScene.removeUser(this.surfaceUsers, surface, entry));
  }

  private static addUser(users: Map<string, Set<ISceneObject>>, key: string, entry: ISceneObject): void {
    let set: Maybe<Set<ISceneObject>> = users.get(key);

    if (!set) {
      set = new Set();
      users.set(key, set);
    }

    set.add(entry);
  }

  private static removeUser(users: Map<string, Set<ISceneObject>>, key: string, entry: ISceneObject): void {
    const set: Maybe<Set<ISceneObject>> = users.get(key);

    set?.delete(entry);

    if (set && !set.size) {
      users.delete(key);
    }
  }

  private buildUsers(users: Maybe<ReadonlySet<ISceneObject>>): void {
    users?.forEach((entry: ISceneObject) => this.build(entry));
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
    this.buildUsers(this.surfaceUsers.get(key));

    if (previous) {
      this.retired.add(previous);
    }
  }

  /** What a surface is built from: the surface itself and the wireframe setting, as one comparable string. */
  private toDescription(surface: IRendererSurface): string {
    return `${this.isWireframe ? "wireframe" : "solid"}:${JSON.stringify(surface)}`;
  }

  /** Caches, or disposes past the cache's limit, every retired material nothing draws any more. */
  private retire(): void {
    if (!this.retired.size) {
      return;
    }

    const drawn: Set<Material> = new Set();
    const meshes: Array<Mesh> = this.changes.flatMap((change: ISceneChange) => change.leaving);

    this.objects.forEach((entry: ISceneObject) => PASSES.forEach((pass) => meshes.push(entry.meshes[pass])));

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

  private isReady(material: Material, layout: string): boolean {
    return material === HIDDEN || Boolean(this.ready.get(material)?.has(layout));
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
    const instances: Nullable<IObjectInstances> = this.toInstances(entry, geometry);
    const count: number = Math.max(
      object.surfaces.length,
      ...geometry.groups.map((group) => (group.materialIndex ?? 0) + 1)
    );
    const surfaces: Array<Maybe<ISurfaceMaterial>> = Array.from({ length: count }, (_, slot: number) =>
      this.surfaces.get(object.surfaces[slot])
    );

    function toSlots(pass: ESurfacePass): Array<Material> {
      return surfaces.map((surface: Maybe<ISurfaceMaterial>) =>
        surface && surface.pass === pass ? surface.material : HIDDEN
      );
    }

    return {
      geometry,
      instances,
      layout: RendererScene.toLayout(geometry, skeleton, instances !== null),
      skeleton,
      slots: {
        [ESurfacePass.DEFERRED]: toSlots(ESurfacePass.DEFERRED),
        [ESurfacePass.FORWARD]: toSlots(ESurfacePass.FORWARD),
        [ESurfacePass.WALLMARK]: toSlots(ESurfacePass.WALLMARK),
      },
    };
  }

  /**
   * The places an object stands, kept while it is put with the same transforms, with its instances' hemisphere terms
   * set on its geometry: a geometry stood in many places is its one object's.
   */
  private toInstances(entry: ISceneObject, geometry: BufferGeometry): Nullable<IObjectInstances> {
    const source: Maybe<IRendererInstances> = entry.object.instances;

    if (!source) {
      return null;
    }

    const instances: IObjectInstances =
      entry.instances?.source === source
        ? entry.instances
        : { matrices: new InstancedBufferAttribute(source.transforms, FLOATS_PER_INSTANCE), source };

    if (source.hemi && geometry.getAttribute(INSTANCE_HEMI_ATTRIBUTE)?.array !== source.hemi) {
      geometry.setAttribute(INSTANCE_HEMI_ATTRIBUTE, new InstancedBufferAttribute(source.hemi, 2));
    }

    return instances;
  }

  /** Queues an object's change in the running transaction's, bringing along whatever it already waited in. */
  private build(entry: ISceneObject): void {
    // Only an object still held draws: one released while it waited is gone for good.
    if (this.objects.get(entry.key) !== entry) {
      return;
    }

    const into: ISceneChange = this.current;

    if (entry.change && entry.change !== into) {
      this.merge(entry.change, into);
    }

    entry.change = into;
    into.objects.add(entry);
  }

  /** Makes an earlier change part of a later one, so neither applies without the other. */
  private merge(from: ISceneChange, into: ISceneChange): void {
    for (const entry of from.objects) {
      entry.change = into;
      into.objects.add(entry);
    }

    into.leaving.push(...from.leaving);
    from.geometries.forEach((geometry: BufferGeometry) => into.geometries.add(geometry));
    from.textures.forEach((key: string) => into.textures.add(key));
    this.changes.splice(this.changes.indexOf(from), 1);
  }

  /**
   * Applies every change that can draw now, oldest first, stopping at the first that cannot: what a consumer changed
   * together appears together, and what it released goes in the same frame.
   */
  private settle(): void {
    while (this.changes.length && this.canApply(this.changes[0])) {
      const change: ISceneChange = this.changes.shift() as ISceneChange;

      change.objects.forEach((entry: ISceneObject) => {
        entry.change = null;
        this.apply(entry);
      });
      change.leaving.forEach((mesh: Mesh) => mesh.removeFromParent());
      change.geometries.forEach((geometry: BufferGeometry) => geometry.dispose());
      change.textures.forEach((key: string) => this.textures.release(key));
    }

    this.retire();
  }

  private canApply(change: ISceneChange): boolean {
    for (const entry of change.objects) {
      const state: Nullable<IObjectState> = this.toState(entry);

      if (state && PASSES.some((pass) => state.slots[pass].some((it) => !this.isReady(it, state.layout)))) {
        return false;
      }
    }

    return true;
  }

  /** A geometry no longer put under its key, disposed once the change replacing it applies. */
  private leaveGeometry(key: string): void {
    const geometry: Maybe<BufferGeometry> = this.geometries.get(key);

    if (geometry) {
      this.current.geometries.add(geometry);
    }
  }

  /** Points every mesh at what the object names now, and shows each only where it has a slot to draw. */
  private apply(entry: ISceneObject): void {
    const state: Nullable<IObjectState> = this.toState(entry);

    if (!state) {
      PASSES.forEach((pass) => entry.meshes[pass].removeFromParent());

      return;
    }

    const { object } = entry;

    if (state.skeleton !== entry.skeleton || state.instances !== entry.instances) {
      PASSES.forEach((pass) => entry.meshes[pass].removeFromParent());
      entry.meshes = RendererScene.createMeshes(state.skeleton, state.instances);
      entry.skeleton = state.skeleton;
      entry.instances = state.instances;
    }

    state.geometry.setDrawRange(object.drawRange?.start ?? 0, object.drawRange?.count ?? Infinity);

    for (const pass of PASSES) {
      const mesh: Mesh = entry.meshes[pass];
      const materials: Array<Material> = state.slots[pass];

      mesh.geometry = state.geometry;
      mesh.material = materials;

      if (object.matrix) {
        mesh.matrix.fromArray(object.matrix);
      } else {
        mesh.matrix.identity();
      }

      mesh.matrixWorldNeedsUpdate = true;

      if (mesh instanceof InstancedMesh) {
        // Measured again over where it stands, lazily, by the first frame that culls it.
        mesh.boundingSphere = null;
      }

      if (materials.some((material: Material) => material !== HIDDEN)) {
        this.scenes[pass].add(mesh);
      } else {
        mesh.removeFromParent();
      }
    }
  }
}
