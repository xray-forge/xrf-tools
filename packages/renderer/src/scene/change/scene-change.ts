import { BufferGeometry, Mesh } from "three/webgpu";

/**
 * What a consumer changed together: applied together, once every object in it can draw, after every change made
 * before it.
 */
export interface ISceneChange<T> {
  /** The objects it brings, each drawn as last put once it applies. */
  objects: Set<T>;
  /** Released objects' meshes, drawn until the change applies, so a replacement never leaves a gap. */
  leaving: Array<Mesh>;
  /** Replaced or released geometries, disposed once nothing draws them. */
  geometries: Set<BufferGeometry>;
  /** Textures released, let go of once whatever sampled them stops drawing. */
  textures: Set<string>;
}
