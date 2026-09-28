import { Node } from "three/webgpu";

/** Each stage's fragment, in the order they run. */
export interface ISmaaStages {
  edges: Node<"vec4">;
  weights: Node<"vec4">;
  blend: Node<"vec4">;
}
