import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Material, Mesh, Scene } from "three/webgpu";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererObject } from "#/contract/scene/renderer-object";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { SceneChangeQueue } from "#/scene/change/scene-change-queue";
import { createRendererBufferGeometry } from "#/scene/geometry/renderer-buffer-geometry";
import { KeyedUsers } from "#/scene/keyed-users";
import { SceneObject } from "#/scene/object/scene-object";
import { SceneObjectResolver } from "#/scene/object/scene-object-resolver";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";
import { createSceneStaging, ISceneStaging } from "#/scene/staging/scene-staging";
import { MaterialReadiness } from "#/scene/surface/material-readiness";
import { SurfaceLibrary } from "#/scene/surface/surface-library";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * Everything a consumer put, as the scenes the frame draws.
 * A change compiles off the frame: until its materials are ready and its textures are up, whatever drew before keeps
 * drawing.
 */
export class RendererScene {
  /** What each pass draws. */
  public readonly scenes: TPassRecord<Scene> = toPassRecord(() => new Scene());
  public readonly textures: RendererTextures;
  public readonly skeletons: RendererSkeletons;

  private readonly geometries: Map<string, BufferGeometry> = new Map();
  private readonly surfaces: SurfaceLibrary;
  private readonly objects: Map<string, SceneObject> = new Map();
  private readonly geometryUsers: KeyedUsers<SceneObject> = new KeyedUsers();
  private readonly surfaceUsers: KeyedUsers<SceneObject> = new KeyedUsers();
  private readonly skeletonUsers: KeyedUsers<SceneObject> = new KeyedUsers();
  private readonly readiness: MaterialReadiness = new MaterialReadiness();
  private readonly resolver: SceneObjectResolver;
  private readonly changes: SceneChangeQueue<SceneObject> = new SceneChangeQueue({
    apply: (entry: SceneObject) => entry.apply(this.resolver.resolve(entry), this.scenes),
    canApply: (entry: SceneObject) => this.canApply(entry),
    releaseTexture: (key: string) => this.textures.release(key),
    settled: () => this.retire(),
  });

  public constructor(uniforms: RendererUniforms, onTextureRefused: (key: string, refusal: IDdsRefusal) => void) {
    this.textures = new RendererTextures(onTextureRefused);
    this.skeletons = new RendererSkeletons((key: string) => this.buildUsers(this.skeletonUsers.get(key)));
    this.surfaces = new SurfaceLibrary(this.textures, uniforms, (key: string) =>
      this.buildUsers(this.surfaceUsers.get(key))
    );
    this.resolver = new SceneObjectResolver(this.geometries, this.skeletons, this.surfaces);
  }

  /** Whether any object waits: for a material to compile, a texture to upload, or its turn to be applied. */
  public get hasPending(): boolean {
    return this.changes.hasPending;
  }

  /**
   * Makes a run of changes one change: it applies only once all of it can draw.
   *
   * @param change - What to change.
   */
  public transact(change: () => void): void {
    this.changes.transact(change);
  }

  /** Applies what became ready since the last frame, a budget of it at a time. */
  public advance(): void {
    this.changes.advance();
  }

  /**
   * @param isWireframe - Whether every surface draws as its triangles' edges.
   */
  public setWireframe(isWireframe: boolean): void {
    this.transact(() => this.surfaces.setWireframe(isWireframe));
  }

  /**
   * Stands the waiting objects with something to compile in scenes of their own, for the renderer to compile. Only
   * once their textures are up: a pipeline built while its samplers held placeholders would not be the one its first
   * frame draws with.
   *
   * @returns The staging, or null when nothing waits to compile.
   */
  public stage(): Nullable<ISceneStaging> {
    const states: Array<ISceneObjectState> = [];

    for (const entry of this.changes.pending) {
      const state: Nullable<ISceneObjectState> = this.resolver.resolve(entry);

      if (state && !this.readiness.isStateReady(state) && this.isUploaded(state)) {
        states.push(state);
      }
    }

    return states.length ? createSceneStaging(states, this.readiness) : null;
  }

  /**
   * Marks what a staging compiled as ready, and applies every change that can draw now.
   *
   * @param staging - What was compiled.
   */
  public commit(staging: ISceneStaging): void {
    staging.materials.forEach(([material, layout]) => this.readiness.mark(material, layout));
    this.transact(() => {});
  }

  public putTexture(key: string, source: TRendererTextureSource): void {
    this.transact(() => {
      this.changes.keepTexture(key);
      this.textures.put(key, source);
    });
  }

  public releaseTexture(key: string): void {
    this.transact(() => this.changes.releaseTexture(key));
  }

  public putGeometry(key: string, geometry: IRendererGeometry): void {
    this.transact(() => {
      this.retireGeometry(key);
      this.geometries.set(key, createRendererBufferGeometry(geometry));
      this.buildUsers(this.geometryUsers.get(key));
    });
  }

  public releaseGeometry(key: string): void {
    this.transact(() => {
      this.retireGeometry(key);
      this.geometries.delete(key);
      this.buildUsers(this.geometryUsers.get(key));
    });
  }

  public putSurface(key: string, surface: IRendererSurface): void {
    this.transact(() => this.surfaces.put(key, surface));
  }

  public releaseSurface(key: string): void {
    this.transact(() => this.surfaces.release(key));
  }

  public putObject(key: string, object: IRendererObject): void {
    this.transact(() => {
      let entry: Maybe<SceneObject> = this.objects.get(key);

      if (entry) {
        this.unindex(entry);
        entry.object = object;
      } else {
        entry = new SceneObject(key, object);
        this.objects.set(key, entry);
      }

      this.index(entry);
      this.build(entry);
    });
  }

  public releaseObject(key: string): void {
    const entry: Maybe<SceneObject> = this.objects.get(key);

    if (entry) {
      this.transact(() => {
        this.unindex(entry);
        this.objects.delete(key);
        this.changes.withdraw(entry, entry.drawing, entry.owned);
      });
    }
  }

  public dispose(): void {
    this.objects.forEach((entry: SceneObject) => entry.detach());
    this.objects.clear();
    this.changes.dispose();
    this.surfaces.dispose();
    this.geometryUsers.clear();
    this.surfaceUsers.clear();
    this.skeletonUsers.clear();
    this.geometries.forEach((geometry: BufferGeometry) => geometry.dispose());
    this.geometries.clear();
    this.skeletons.dispose();
    this.textures.dispose();
  }

  private index(entry: SceneObject): void {
    const { geometry, surfaces, skeleton } = entry.object;

    this.geometryUsers.add(geometry, entry);
    surfaces.forEach((surface: string) => this.surfaceUsers.add(surface, entry));

    if (skeleton) {
      this.skeletonUsers.add(skeleton, entry);
    }
  }

  private unindex(entry: SceneObject): void {
    const { geometry, surfaces, skeleton } = entry.object;

    this.geometryUsers.delete(geometry, entry);
    surfaces.forEach((surface: string) => this.surfaceUsers.delete(surface, entry));

    if (skeleton) {
      this.skeletonUsers.delete(skeleton, entry);
    }
  }

  private buildUsers(users: ReadonlySet<SceneObject>): void {
    this.transact(() => users.forEach((entry: SceneObject) => this.build(entry)));
  }

  /** Queues an object in the running change, to draw as it is put now. */
  private build(entry: SceneObject): void {
    // Only an object still held draws: one released while it waited is gone for good.
    if (this.objects.get(entry.key) === entry) {
      this.changes.enlist(entry);
    }
  }

  /** A geometry no longer put under its key, disposed once the change replacing it applies. */
  private retireGeometry(key: string): void {
    const geometry: Maybe<BufferGeometry> = this.geometries.get(key);

    if (geometry) {
      this.changes.retireGeometry(geometry);
    }
  }

  /** Whether an object draws without a stall: its materials compiled, its textures uploaded. */
  private canApply(entry: SceneObject): boolean {
    const state: Nullable<ISceneObjectState> = this.resolver.resolve(entry);

    // One whose geometry is missing applies at once, as nothing drawn.
    return !state || (this.readiness.isStateReady(state) && this.isUploaded(state));
  }

  private isUploaded(state: ISceneObjectState): boolean {
    return state.keys.every((key: string) => this.textures.isUploaded(key));
  }

  /** Lets the surface library cache or dispose whatever it retired that nothing draws any more. */
  private retire(): void {
    if (!this.surfaces.hasRetired) {
      return;
    }

    const drawn: Set<Material> = new Set();
    const meshes: Array<Mesh> = [...this.changes.leaving];

    this.objects.forEach((entry: SceneObject) => meshes.push(...entry.drawing));

    for (const mesh of meshes) {
      (Array.isArray(mesh.material) ? mesh.material : [mesh.material]).forEach((it: Material) => drawn.add(it));
    }

    this.surfaces.retire(drawn, (material: Material) => this.readiness.isCompiled(material));
  }
}
