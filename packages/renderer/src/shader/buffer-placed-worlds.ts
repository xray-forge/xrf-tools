import { Node } from "three/webgpu";

/** Where a static draw's vertex stands in the world this frame, and where it stood the frame before. */
export interface IBufferPlacedWorlds {
  readonly current: Node<"vec3">;
  readonly previous: Node<"vec3">;
}
