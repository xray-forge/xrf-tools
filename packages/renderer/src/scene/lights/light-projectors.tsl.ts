import { vec2 } from "three/tsl";
import { Node } from "three/webgpu";

/** Where a projector's sampler is bound: each spot samples it again at its own coordinates. */
export function toProjectorAnchor(): Node<"vec2"> {
  return vec2(0, 0);
}
