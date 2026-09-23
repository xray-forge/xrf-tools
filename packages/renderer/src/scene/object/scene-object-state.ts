import { Nullable } from "@xrf/types";
import { BufferGeometry, Material, Skeleton } from "three/webgpu";

import { SceneInstances } from "#/scene/object/scene-instances";
import { TPassRecord } from "#/scene/pass-record";

/**
 * What an object draws as it is put now, once every material in it is compiled for its layout.
 */
export interface ISceneObjectState {
  /** What its meshes draw: the geometry put, or its instanced geometry. */
  geometry: BufferGeometry;
  skeleton: Nullable<Skeleton>;
  instances: Nullable<SceneInstances>;
  /** Each pass's material for every slot, hidden where the pass draws none of it. */
  slots: TPassRecord<Array<Material>>;
  /** The vertex layout the materials compile against, which three builds a shader per. */
  layout: string;
  /** Every texture key its surfaces sample. */
  keys: ReadonlyArray<string>;
}
