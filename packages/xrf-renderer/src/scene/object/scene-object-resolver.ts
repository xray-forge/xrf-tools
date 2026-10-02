import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Skeleton } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { ISceneSection } from "#/scene/geometry/scene-section";
import { RendererImpostorSets } from "#/scene/impostor/renderer-impostor-sets";
import { SceneInstances } from "#/scene/object/scene-instances";
import { SceneObject } from "#/scene/object/scene-object";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { RendererSkeletonEntry } from "#/scene/skeleton/renderer-skeleton-entry";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";
import { StaticDraws } from "#/scene/static/static-draws";
import { SurfaceLibrary } from "#/scene/surface/surface-library";

/**
 * Works out what an object draws from what it names: its geometry, its skeleton, its places and its surfaces. What it
 * names changes only through the scene building the object again, which forgets what it came to, so an object waiting
 * to draw is resolved once rather than on every frame it waits.
 */
export class SceneObjectResolver {
  private readonly geometries: ReadonlyMap<string, SceneGeometry>;
  private readonly skeletons: RendererSkeletons;
  private readonly surfaces: SurfaceLibrary;
  private readonly impostors: RendererImpostorSets;
  private readonly draws: StaticDraws;
  /** Each geometry's attribute names, sorted. */
  private readonly attributeNames: WeakMap<BufferGeometry, string> = new WeakMap();
  /** What each object came to since it was last built. */
  private readonly resolved: WeakMap<SceneObject, Nullable<ISceneObjectState>> = new WeakMap();

  public constructor(
    geometries: ReadonlyMap<string, SceneGeometry>,
    skeletons: RendererSkeletons,
    surfaces: SurfaceLibrary,
    impostors: RendererImpostorSets,
    draws: StaticDraws
  ) {
    this.geometries = geometries;
    this.skeletons = skeletons;
    this.surfaces = surfaces;
    this.impostors = impostors;
    this.draws = draws;
  }

  /**
   * @param entry - The object.
   * @returns What it would draw now, or null for one whose geometry is missing.
   */
  public resolve(entry: SceneObject): Nullable<ISceneObjectState> {
    let state: Maybe<Nullable<ISceneObjectState>> = this.resolved.get(entry);

    if (state === undefined) {
      state = this.toState(entry);
      this.resolved.set(entry, state);
    }

    return state;
  }

  /**
   * Forgets what an object came to, as the scene builds it again: what it names, or anything named, changed.
   *
   * @param entry - The object.
   */
  public forget(entry: SceneObject): void {
    this.resolved.delete(entry);
  }

  private toState(entry: SceneObject): Nullable<ISceneObjectState> {
    const { object } = entry;
    const geometry: Maybe<SceneGeometry> = this.geometries.get(object.geometry);

    if (!geometry) {
      return null;
    }

    // Skinned only when the object names a skeleton that exists and its geometry carries the links to bind with.
    const skeletonEntry: Maybe<RendererSkeletonEntry> = object.skeleton
      ? this.skeletons.get(object.skeleton)
      : undefined;
    const skeleton: Nullable<Skeleton> =
      skeletonEntry && geometry.buffer.hasAttribute("skinIndex") ? skeletonEntry.skeleton : null;
    const instances: Nullable<SceneInstances> = entry.toInstances(geometry);
    const drawn: BufferGeometry = instances?.geometry ?? geometry.buffer;
    // Unskinned, in a layout an arena stores, it stands its clusters in a place of its own or in each of its places,
    // which the buffers hold; its static draws' materials compile against its arena's prototype.
    const staticDrawn: Nullable<BufferGeometry> =
      this.draws.isEnabled && !skeleton ? (this.draws.toArena(geometry)?.prototype ?? null) : null;
    // A static draw draws by the material its surface shares with others where it has one.
    const surfaces: Array<Maybe<ISurfaceMaterial>> = geometry.sections.map((section: ISceneSection) => {
      const surface: Maybe<ISurfaceMaterial> = this.surfaces.get(object.surfaces[section.slot]);

      return (staticDrawn && surface?.batched) || surface;
    });

    return {
      geometry,
      instances,
      keys: surfaces.flatMap((surface: Maybe<ISurfaceMaterial>) => surface?.keys ?? []),
      lodStart: instances ? this.impostors.getStart(object.instances?.impostors?.key) : null,
      plain: { drawn, layout: this.toLayout(drawn, skeleton, instances !== null) },
      skeleton,
      static: staticDrawn ? { drawn: staticDrawn, layout: this.toLayout(staticDrawn, null, false) } : null,
      surfaces,
    };
  }

  /** The vertex layout a mesh compiles against, which three builds a shader per. */
  private toLayout(geometry: BufferGeometry, skeleton: Nullable<Skeleton>, isInstanced: boolean): string {
    let names: Maybe<string> = this.attributeNames.get(geometry);

    if (names === undefined) {
      names = Object.keys(geometry.attributes).sort().join(",");
      this.attributeNames.set(geometry, names);
    }

    return `${isInstanced ? "instanced" : ""}${skeleton ? "skinned" : ""}:${names}`;
  }
}
