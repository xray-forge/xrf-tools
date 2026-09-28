import { Maybe, Nullable } from "@xrf/types";
import { Material, Mesh, Object3D, PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";

import { IRendererStaticDrawReport } from "#/contract/renderer-static-draw-report";
import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererGrass } from "#/contract/scene/renderer-grass";
import { IRendererImpostors } from "#/contract/scene/renderer-impostors";
import { IRendererLights } from "#/contract/scene/renderer-lights";
import { IRendererObject } from "#/contract/scene/renderer-object";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IDdsRefusal } from "#/dds/dds-refusal";
import { SceneChangeQueue } from "#/scene/change/scene-change-queue";
import { GeometryReleases } from "#/scene/geometry/geometry-releases";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { SceneGrass } from "#/scene/grass/scene-grass";
import { RendererImpostorSets } from "#/scene/impostor/renderer-impostor-sets";
import { KeyedUsers } from "#/scene/keyed-users";
import { SceneLights } from "#/scene/lights/scene-lights";
import { createSceneRoot } from "#/scene/object/scene-mesh";
import { SceneObject } from "#/scene/object/scene-object";
import { SceneObjectResolver } from "#/scene/object/scene-object-resolver";
import { ISceneObjectState, isStaticDraw } from "#/scene/object/scene-object-state";
import { toPassRecord, TPassRecord } from "#/scene/pass-record";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";
import { SceneSky } from "#/scene/sky/scene-sky";
import { LayoutProxies } from "#/scene/staging/layout-proxies";
import { createSceneStaging, ISceneStaging } from "#/scene/staging/scene-staging";
import { StaticCull } from "#/scene/static/static-cull";
import { StaticDraws } from "#/scene/static/static-draws";
import { IStaticShadowCasters } from "#/scene/static/static-shadow-casters";
import { IStaticUpcoming } from "#/scene/static/static-upcoming";
import { MaterialReadiness } from "#/scene/surface/material-readiness";
import { SurfaceLibrary } from "#/scene/surface/surface-library";
import { RendererTextures } from "#/texture/renderer-textures";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { CullView } from "#/visibility/cull-view";

/**
 * Everything a consumer put, as the scenes the frame draws.
 * A change compiles off the frame: until its materials are ready and its textures are up, whatever drew before keeps
 * drawing.
 */
export class RendererScene {
  /** What each pass draws. Its meshes keep their own matrices current, so three never walks them to. */
  public readonly scenes: TPassRecord<Scene> = toPassRecord(createSceneRoot);
  public readonly textures: RendererTextures;
  public readonly skeletons: RendererSkeletons;
  /** What culls the static draws on the GPU, which the frame dispatches before drawing them. */
  public readonly staticCull: StaticCull;
  /** The level's grass, planted on the GPU by the grass pass. */
  public readonly grass: SceneGrass;
  /** The local lights, written out each frame for the lights pass. */
  public readonly lights: SceneLights;
  /** The skies the lighting names, which the water reflects. */
  public readonly sky: SceneSky;

  private readonly geometries: Map<string, SceneGeometry> = new Map();
  /** The buffers of geometries nothing draws any more, freed with the next flush. */
  private readonly releases: GeometryReleases = new GeometryReleases();
  private readonly surfaces: SurfaceLibrary;
  private readonly objects: Map<string, SceneObject> = new Map();
  private readonly geometryUsers: KeyedUsers<SceneObject> = new KeyedUsers();
  private readonly surfaceUsers: KeyedUsers<SceneObject> = new KeyedUsers();
  private readonly skeletonUsers: KeyedUsers<SceneObject> = new KeyedUsers();
  private readonly impostorUsers: KeyedUsers<SceneObject> = new KeyedUsers();
  private readonly impostors: RendererImpostorSets;
  private readonly staticDraws: StaticDraws;
  private readonly readiness: MaterialReadiness = new MaterialReadiness();
  private readonly proxies: LayoutProxies = new LayoutProxies();
  /** The texture keys each drawn object samples, whose own textures stay on the GPU while it draws them. */
  private readonly held: Map<SceneObject, ReadonlyArray<string>> = new Map();
  private readonly resolver: SceneObjectResolver;
  private readonly changes: SceneChangeQueue<SceneObject> = new SceneChangeQueue({
    apply: (entry: SceneObject) => this.apply(entry),
    canApply: (entry: SceneObject) => this.canApply(entry),
    releaseTexture: (key: string) => this.textures.release(key),
    settled: () => this.retire(),
  });

  public constructor(uniforms: RendererUniforms, onTextureRefused: (key: string, refusal: IDdsRefusal) => void) {
    this.staticDraws = new StaticDraws(uniforms.staticDraws, uniforms.treeWind, () => this.toUpcomingStatic());
    this.staticCull = this.staticDraws.cull;
    this.textures = new RendererTextures(onTextureRefused, (key: string) => {
      this.surfaces.rebind(key);
      this.staticDraws.invalidate(key);
    });
    this.grass = new SceneGrass(this.textures, uniforms);
    this.lights = new SceneLights(this.textures, this.staticDraws.shadowChanges);
    this.sky = new SceneSky(this.textures, uniforms.sky);
    this.skeletons = new RendererSkeletons((key: string, release: Nullable<() => void>) =>
      this.replace(this.skeletonUsers.get(key), release)
    );
    this.surfaces = new SurfaceLibrary(
      this.textures,
      uniforms,
      (key: string, release: Nullable<() => void>) => this.replace(this.surfaceUsers.get(key), release),
      (key: string) => this.staticDraws.invalidate(key)
    );
    this.impostors = new RendererImpostorSets(this.staticDraws, (key: string, release: Nullable<() => void>) =>
      this.replace(this.impostorUsers.get(key), release)
    );
    this.resolver = new SceneObjectResolver(
      this.geometries,
      this.skeletons,
      this.surfaces,
      this.impostors,
      this.staticDraws
    );
  }

  /** What each shadow view draws: every casting static batch, by what its cull kept. */
  public get shadowCasters(): IStaticShadowCasters {
    return this.staticDraws;
  }

  /**
   * @param isEnabled - Whether the device draws static draws: only one drawing an indirect draw's first instance.
   */
  public setStaticDraws(isEnabled: boolean): void {
    if (isEnabled !== this.staticDraws.isEnabled) {
      this.staticDraws.isEnabled = isEnabled;
      this.transact(() => this.objects.forEach((entry: SceneObject) => this.build(entry)));
    }
  }

  /** How full the static draws' pools are and what the last cull found occluded. */
  public get staticDrawReport(): IRendererStaticDrawReport {
    return this.staticDraws.report;
  }

  /** Layers the device allows a texture array, which the device says once it is open. */
  public set arrayLayerLimit(limit: number) {
    this.surfaces.arrayLayerLimit = limit;
  }

  /**
   * Puts what the static batches' shared materials read on the GPU, the layers their arrays wait for and their rows,
   * and frees the buffers of the geometries nothing draws any more.
   *
   * @param renderer - The renderer about to draw.
   */
  public flush(renderer: WebGPURenderer): void {
    this.surfaces.flush(renderer);
    this.releases.free(renderer);
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
   * Shows what a view sees of every object and hides the rest, doing nothing for a view that has not moved.
   *
   * @param view - The view about to be drawn.
   * @param camera - Its camera.
   */
  public cull(view: CullView, camera: PerspectiveCamera): void {
    // The trees and impostors of every clump are chosen on the GPU, in the static cull.
    this.objects.forEach((entry: SceneObject) => entry.cull(view));
    this.staticCull.take(view, camera);
  }

  /**
   * @param isWireframe - Whether every surface draws as its triangles' edges.
   */
  public setWireframe(isWireframe: boolean): void {
    this.surfaces.setWireframe(isWireframe);
    this.staticDraws.setWireframe(isWireframe ? this.surfaces.wireframeMaterial.material : null);
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

    return states.length ? createSceneStaging(states, this.readiness, this.proxies) : null;
  }

  /**
   * Marks what a staging compiled as ready, and applies every change that can draw now.
   *
   * @param staging - What was compiled.
   */
  public commit(staging: ISceneStaging): void {
    staging.materials.forEach(([material, layout]: readonly [Material, string]) =>
      this.readiness.mark(material, layout)
    );
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
    // Made first: one refused leaves the geometry it would have replaced drawn, and nothing queued.
    const entry: SceneGeometry = new SceneGeometry(geometry);

    this.transact(() => {
      const release: Nullable<() => void> = this.toGeometryRelease(key);

      this.geometries.set(key, entry);
      this.replace(this.geometryUsers.get(key), release);
    });
  }

  public releaseGeometry(key: string): void {
    this.transact(() => {
      const release: Nullable<() => void> = this.toGeometryRelease(key);

      this.geometries.delete(key);
      this.replace(this.geometryUsers.get(key), release);
    });
  }

  /**
   * @param grass - A level's grass, replacing any put before.
   */
  public putGrass(grass: IRendererGrass): void {
    this.grass.put(grass);
  }

  public releaseGrass(): void {
    this.grass.release();
  }

  /**
   * @param lights - The scene's local lights, replacing any put before.
   */
  public putLights(lights: IRendererLights): void {
    this.lights.put(lights);
  }

  public releaseLights(): void {
    this.lights.release();
  }

  public putImpostors(key: string, impostors: IRendererImpostors): void {
    this.transact(() => this.impostors.put(key, impostors));
  }

  public releaseImpostors(key: string): void {
    this.transact(() => this.impostors.release(key));
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
        entry = new SceneObject(key, object, this.staticDraws, this.releases);
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
        this.changes.withdraw(entry, entry.placed, () => {
          entry.dispose();
          this.hold(entry, []);
        });
      });
    }
  }

  public dispose(): void {
    this.grass.dispose();
    this.lights.dispose();
    this.sky.dispose();
    this.objects.forEach((entry: SceneObject) => entry.dispose());
    this.objects.clear();
    this.changes.dispose();
    this.surfaces.dispose();
    this.geometryUsers.clear();
    this.surfaceUsers.clear();
    this.skeletonUsers.clear();
    this.impostorUsers.clear();
    this.impostors.dispose();
    this.geometries.forEach((geometry: SceneGeometry) => geometry.dispose(this.releases));
    this.geometries.clear();
    this.skeletons.dispose();
    this.held.clear();
    this.textures.dispose();
    this.staticDraws.dispose();
    this.proxies.dispose();
  }

  private index(entry: SceneObject): void {
    const { geometry, surfaces, skeleton, instances } = entry.object;

    this.geometryUsers.add(geometry, entry);
    surfaces.forEach((surface: string) => this.surfaceUsers.add(surface, entry));

    if (skeleton) {
      this.skeletonUsers.add(skeleton, entry);
    }

    if (instances?.impostors) {
      this.impostorUsers.add(instances.impostors.key, entry);
    }
  }

  private unindex(entry: SceneObject): void {
    const { geometry, surfaces, skeleton, instances } = entry.object;

    this.geometryUsers.delete(geometry, entry);
    surfaces.forEach((surface: string) => this.surfaceUsers.delete(surface, entry));

    if (skeleton) {
      this.skeletonUsers.delete(skeleton, entry);
    }

    if (instances?.impostors) {
      this.impostorUsers.delete(instances.impostors.key, entry);
    }
  }

  private buildUsers(users: ReadonlySet<SceneObject>): void {
    this.transact(() => users.forEach((entry: SceneObject) => this.build(entry)));
  }

  /**
   * Rebuilds the users of a keyed resource put again or released, in the change that lets the one it replaced go.
   *
   * @param users - The objects naming its key.
   * @param release - What lets the replaced one go, or null for none.
   */
  private replace(users: ReadonlySet<SceneObject>, release: Nullable<() => void>): void {
    this.transact(() => {
      if (release) {
        this.changes.retire(release);
      }

      this.buildUsers(users);
    });
  }

  /** Queues an object in the running change, to draw as it is put now. */
  private build(entry: SceneObject): void {
    // Only an object still held draws: one released while it waited is gone for good.
    if (this.objects.get(entry.key) === entry) {
      this.changes.enlist(entry);
    }
  }

  /**
   * @param key - A geometry's key, about to be put again or released.
   * @returns What frees the geometry it holds now, or null for none.
   */
  private toGeometryRelease(key: string): Nullable<() => void> {
    const geometry: Maybe<SceneGeometry> = this.geometries.get(key);

    return geometry ? () => geometry.dispose(this.releases) : null;
  }

  /** Draws an object as it is put now. */
  private apply(entry: SceneObject): void {
    const state: Nullable<ISceneObjectState> = this.resolver.resolve(entry);

    entry.apply(state, this.scenes);
    this.hold(entry, state ? [...new Set([...state.keys, ...entry.plainKeys])] : []);
  }

  /**
   * Keeps the own textures of what an object draws now on the GPU, whatever the arrays hold, and lets go of what it
   * drew before: its static batches' keys, and its parts drawn plainly, a part refused a static draw among them.
   */
  private hold(entry: SceneObject, keys: ReadonlyArray<string>): void {
    this.textures.hold(keys);
    this.textures.letGo(this.held.get(entry) ?? []);

    if (keys.length) {
      this.held.set(entry, keys);
    } else {
      this.held.delete(entry);
    }
  }

  /** What the waiting objects will take of the static draws. */
  private *toUpcomingStatic(): Iterable<IStaticUpcoming> {
    for (const entry of this.changes.pending) {
      const state: Nullable<ISceneObjectState> = this.resolver.resolve(entry);

      if (!state) {
        continue;
      }

      const sections: number = state.geometry.sections.filter((_: unknown, index: number) =>
        isStaticDraw(state, state.surfaces[index])
      ).length;

      if (sections) {
        yield {
          clusters: state.geometry.clusters.count,
          geometry: state.geometry,
          places: state.instances?.places ?? 0,
          sections,
        };
      }
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
    const meshes: Array<Mesh> = [];

    for (const leaving of this.changes.leaving) {
      leaving.traverse((it: Object3D) => it instanceof Mesh && meshes.push(it));
    }

    // Placed only: a part's mesh out of every scene still names the material it was last shown with.
    this.objects.forEach((entry: SceneObject) => meshes.push(...entry.placed));

    for (const mesh of meshes) {
      (Array.isArray(mesh.material) ? mesh.material : [mesh.material]).forEach((it: Material) => drawn.add(it));
    }

    for (const material of this.staticDraws.materials) {
      drawn.add(material);
    }

    this.surfaces.retire(drawn, (material: Material) => this.readiness.isCompiled(material));
  }
}
