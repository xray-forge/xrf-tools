import { Maybe, Nullable } from "@xrf/types";
import { Skeleton } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { SceneInstances } from "#/scene/object/scene-instances";
import { ISceneObjectDraw, isStaticSurface } from "#/scene/object/scene-object-draw";

/**
 * What an object draws as it is put now, once every material in it is compiled for its layout.
 */
export interface ISceneObjectState {
  geometry: SceneGeometry;
  skeleton: Nullable<Skeleton>;
  instances: Nullable<SceneInstances>;
  /** How its parts draw plainly: over the geometry's own buffer, or the one standing it in every place. */
  plain: ISceneObjectDraw;
  /** How its parts draw as static draws, for an object neither instanced nor skinned; null otherwise. */
  static: Nullable<ISceneObjectDraw>;
  /** Each section's surface, by the section's position; missing where the object names none that is put. */
  surfaces: ReadonlyArray<Maybe<ISurfaceMaterial>>;
  /** Every texture key its surfaces sample. */
  keys: ReadonlyArray<string>;
  /** Where the impostors its places belong to start in the LOD pool, or null where they belong to none there is. */
  lodStart: Nullable<number>;
}

/**
 * @param state - What an object draws.
 * @param surface - One of its surfaces, or none for a section naming a surface not put.
 * @returns Whether the parts that surface draws are static draws.
 */
export function isStaticDraw(state: ISceneObjectState, surface: Maybe<ISurfaceMaterial>): surface is ISurfaceMaterial {
  return state.static !== null && Boolean(surface) && isStaticSurface(surface as ISurfaceMaterial);
}

/**
 * @param state - What an object draws.
 * @param surface - One of its surfaces.
 * @returns How the parts that surface draws are drawn, which is what its material compiles for.
 */
export function toSurfaceDraw(state: ISceneObjectState, surface: ISurfaceMaterial): ISceneObjectDraw {
  return state.static && isStaticDraw(state, surface) ? state.static : state.plain;
}
