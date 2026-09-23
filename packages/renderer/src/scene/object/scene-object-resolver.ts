import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Skeleton } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { ISceneSection } from "#/scene/geometry/scene-section";
import { SceneInstances } from "#/scene/object/scene-instances";
import { SceneObject } from "#/scene/object/scene-object";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { RendererSkeletonEntry } from "#/scene/skeleton/renderer-skeleton-entry";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";
import { StaticDraws } from "#/scene/static/static-draws";
import { SurfaceLibrary } from "#/scene/surface/surface-library";

/**
 * Works out what an object draws from what it names: its geometry, its skeleton, its places and its surfaces.
 */
export class SceneObjectResolver {
  /** The vertex layout a mesh compiles against, which three builds a shader per. */
  private static toLayout(geometry: BufferGeometry, skeleton: Nullable<Skeleton>, isInstanced: boolean): string {
    return `${isInstanced ? "instanced" : ""}${skeleton ? "skinned" : ""}:${Object.keys(geometry.attributes).sort()}`;
  }

  private readonly geometries: ReadonlyMap<string, SceneGeometry>;
  private readonly skeletons: RendererSkeletons;
  private readonly surfaces: SurfaceLibrary;
  private readonly draws: StaticDraws;

  public constructor(
    geometries: ReadonlyMap<string, SceneGeometry>,
    skeletons: RendererSkeletons,
    surfaces: SurfaceLibrary,
    draws: StaticDraws
  ) {
    this.geometries = geometries;
    this.skeletons = skeletons;
    this.surfaces = surfaces;
    this.draws = draws;
  }

  /**
   * @param entry - The object.
   * @returns What it would draw now, or null for one whose geometry is missing.
   */
  public resolve(entry: SceneObject): Nullable<ISceneObjectState> {
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
    const surfaces: Array<Maybe<ISurfaceMaterial>> = geometry.sections.map((section: ISceneSection) =>
      this.surfaces.get(object.surfaces[section.slot])
    );

    // Neither instanced nor skinned, it stands where its matrix puts it, which a static draw can say by its slot; its
    // static draws' materials compile against its arena's prototype.
    const staticDrawn: Nullable<BufferGeometry> =
      this.draws.isEnabled && !instances && !skeleton ? this.draws.toArena(geometry).prototype : null;

    return {
      geometry,
      instances,
      keys: surfaces.flatMap((surface: Maybe<ISurfaceMaterial>) => surface?.keys ?? []),
      plain: { drawn, layout: SceneObjectResolver.toLayout(drawn, skeleton, instances !== null) },
      skeleton,
      static: staticDrawn
        ? { drawn: staticDrawn, layout: SceneObjectResolver.toLayout(staticDrawn, null, false) }
        : null,
      surfaces,
    };
  }
}
