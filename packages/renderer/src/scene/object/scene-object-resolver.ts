import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Material, Skeleton } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneInstances } from "#/scene/object/scene-instances";
import { SceneObject } from "#/scene/object/scene-object";
import { ISceneObjectState } from "#/scene/object/scene-object-state";
import { toPassRecord } from "#/scene/pass-record";
import { RendererSkeletonEntry } from "#/scene/skeleton/renderer-skeleton-entry";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";
import { HIDDEN_MATERIAL } from "#/scene/surface/hidden-material";
import { SurfaceLibrary } from "#/scene/surface/surface-library";

/**
 * Works out what an object draws from what it names: its geometry, its skeleton, its places and its surfaces.
 */
export class SceneObjectResolver {
  /** The vertex layout a mesh compiles against, which three builds a shader per. */
  private static toLayout(geometry: BufferGeometry, skeleton: Nullable<Skeleton>, isInstanced: boolean): string {
    return `${isInstanced ? "instanced" : ""}${skeleton ? "skinned" : ""}:${Object.keys(geometry.attributes).sort()}`;
  }

  private readonly geometries: ReadonlyMap<string, BufferGeometry>;
  private readonly skeletons: RendererSkeletons;
  private readonly surfaces: SurfaceLibrary;

  public constructor(
    geometries: ReadonlyMap<string, BufferGeometry>,
    skeletons: RendererSkeletons,
    surfaces: SurfaceLibrary
  ) {
    this.geometries = geometries;
    this.skeletons = skeletons;
    this.surfaces = surfaces;
  }

  /**
   * @param entry - The object.
   * @returns What it would draw now, or null for one whose geometry is missing.
   */
  public resolve(entry: SceneObject): Nullable<ISceneObjectState> {
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
    const instances: Nullable<SceneInstances> = entry.toInstances(geometry);
    const drawn: BufferGeometry = instances?.geometry ?? geometry;
    const count: number = Math.max(
      object.surfaces.length,
      ...geometry.groups.map((group) => (group.materialIndex ?? 0) + 1)
    );
    const surfaces: Array<Maybe<ISurfaceMaterial>> = Array.from({ length: count }, (_, slot: number) =>
      this.surfaces.get(object.surfaces[slot])
    );

    return {
      geometry: drawn,
      instances,
      keys: surfaces.flatMap((surface: Maybe<ISurfaceMaterial>) => surface?.keys ?? []),
      layout: SceneObjectResolver.toLayout(drawn, skeleton, instances !== null),
      skeleton,
      slots: toPassRecord((pass: ERendererPass) =>
        surfaces.map((surface: Maybe<ISurfaceMaterial>): Material =>
          surface?.pass === pass ? surface.material : HIDDEN_MATERIAL
        )
      ),
    };
  }
}
