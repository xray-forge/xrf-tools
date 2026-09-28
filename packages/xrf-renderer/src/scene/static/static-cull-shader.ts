import { ComputeNode, Vector4 } from "three/webgpu";

import { IStaticViewCullShader } from "#/scene/static/static-view-cull-shader";

/**
 * The culls of the static draws, a frame's worth, each built over the buffers as they are laid out and dispatched only
 * as far as their clusters, rows and batches are used.
 */
export interface IStaticCullShader {
  /**
   * The camera's first view, in dispatch order: its arguments and the second view's cleared, the LOD cull, then its
   * single draws' clusters, then its instanced draws' rows and their clusters.
   */
  early: ReadonlyArray<ComputeNode>;
  /** The camera's second view: what the first left as hidden, tested against this frame's depth. */
  late: ComputeNode;
  /** Each of the camera's views' arguments rewritten for its wireframe draw, the first then the second. */
  wire: ReadonlyArray<ComputeNode>;
  /** Six planes, normals pointing in, `w` the constant. */
  planes: ReadonlyArray<Vector4>;
  /** Each shadow view's cull: its arguments cleared, its single draws' clusters, its rows', and the planes they read. */
  views: ReadonlyArray<IStaticViewCullShader>;
}
