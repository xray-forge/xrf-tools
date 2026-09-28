import { Vector4 } from "three/webgpu";

/**
 * What a shadow view's casters are culled by: its planes, and a version bumped whenever they move, so a cull already
 * run against them is not run again.
 */
export interface IShadowFrustum {
  /** Six planes, normals pointing in, `w` the constant. */
  readonly planes: ReadonlyArray<Vector4>;
  readonly version: number;
}
