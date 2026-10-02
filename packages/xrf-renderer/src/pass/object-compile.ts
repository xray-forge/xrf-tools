import { Nullable } from "@xrf/types";
import { Camera, Object3D, RenderTarget, Scene } from "three/webgpu";

/** One object to compile for a target, as a scene drawing it draws it. */
export interface IObjectCompile {
  /** Where it draws: the canvas when null. */
  target: Nullable<RenderTarget>;
  object: Object3D;
  /** The scene it is drawn in, whose lighting and environment its materials build with. */
  scene: Scene;
  camera: Camera;
}
