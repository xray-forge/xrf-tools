import { ComputeNode, Vector4 } from "three/webgpu";

/** One shadow view's cull: no occlusion and no second phase, only its frustum. */
export interface IStaticViewCullShader {
  cull: ReadonlyArray<ComputeNode>;
  planes: ReadonlyArray<Vector4>;
}
