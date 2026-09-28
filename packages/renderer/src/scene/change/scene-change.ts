import { Object3D } from "three/webgpu";

/**
 * What a consumer changed together: applied together, once every object in it can draw, after every change made
 * before it.
 */
export interface ISceneChange<T> {
  /** The objects it brings, each drawn as last put once it applies. */
  objects: Set<T>;
  /** What stood in the scenes for released objects, drawn until the change applies, so a replacement leaves no gap. */
  leaving: Array<Object3D>;
  /** What lets released objects' resources go, once nothing draws them. */
  disposals: Array<() => void>;
  /** What lets go of keyed resources replaced or released, once nothing draws them: after every change before it. */
  retired: Array<() => void>;
  /** Textures released, let go of once whatever sampled them stops drawing. */
  textures: Set<string>;
}
