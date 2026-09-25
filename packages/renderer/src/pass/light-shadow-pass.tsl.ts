import { float, vec4 } from "three/tsl";
import { Node } from "three/webgpu";

/** Reversed depth's far plane, which a square of the atlas is cleared to before its face is drawn. */
export function toFarDepth(): Node<"float"> {
  return float(0);
}

/** What a depth-only draw writes to the colour it cannot go without. */
export function toNoColor(): Node<"vec4"> {
  return vec4(0);
}
