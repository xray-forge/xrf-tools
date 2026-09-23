import { Maybe, Nullable } from "@xrf/types";
import { BufferGeometry, Skeleton } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { SceneInstances } from "#/scene/object/scene-instances";

/**
 * What an object draws as it is put now, once every material in it is compiled for its layout.
 */
export interface ISceneObjectState {
  geometry: SceneGeometry;
  skeleton: Nullable<Skeleton>;
  instances: Nullable<SceneInstances>;
  /** What its parts draw over: the geometry's own buffer, or the one standing it in every place. */
  drawn: BufferGeometry;
  /** Each section's surface, by the section's position; missing where the object names none that is put. */
  surfaces: ReadonlyArray<Maybe<ISurfaceMaterial>>;
  /** The vertex layout the materials compile against, which three builds a shader per. */
  layout: string;
  /** Every texture key its surfaces sample. */
  keys: ReadonlyArray<string>;
}
