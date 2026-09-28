import { Node } from "three/webgpu";

/**
 * One point of a surface, in view space, as the base lighting reads it.
 */
export interface IBaseShadingPoint {
  position: Node<"vec3">;
  /** Unit length. */
  normal: Node<"vec3">;
  /** The lighting model slice, `(class + 0.5) / 4`. */
  slice: Node<"float">;
}
