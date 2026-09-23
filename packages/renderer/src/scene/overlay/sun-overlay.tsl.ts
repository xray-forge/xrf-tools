import { float, uv } from "three/tsl";
import { Node } from "three/webgpu";

/**
 * @returns A sprite's coverage as a disc rather than its square: one inside, zero outside.
 */
export function toSunDiscOpacity(): Node<"float"> {
  return uv().sub(0.5).length().lessThan(0.5).select(float(1), float(0));
}
