import { UniformNode, Vector2, Vector3 } from "three/webgpu";

/** One of a bolt's glows as its shader reads it. */
export interface IThunderGlowUniforms {
  /** In renderer space. */
  position: UniformNode<"vec3", Vector3>;
  /** Half its width and half its height, in metres. */
  extent: UniformNode<"vec2", Vector2>;
  /** What its colour and alpha are both scaled by. */
  opacity: UniformNode<"float", number>;
}
