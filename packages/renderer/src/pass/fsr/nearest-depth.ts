import { Node } from "three/webgpu";

/** The nearest depth about a texel, and the texel it stands at. */
export interface INearestDepth {
  depth: Node<"float">;
  at: Node<"vec2">;
}
