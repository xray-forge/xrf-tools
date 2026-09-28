import { Node } from "three/webgpu";

/** `EPS`, what `CalcSSA` adds to a squared distance so a camera standing at a centre divides by something. */
const DISTANCE_EPSILON: number = 0.00001;

/**
 * @param radius - A sphere's radius.
 * @param offset - Its centre, from the camera.
 * @returns Its screen area as `CalcSSA` takes it: its radius over its squared distance.
 */
export function toStaticScreenArea(radius: Node<"float">, offset: Node<"vec3">): Node<"float"> {
  return radius.div(offset.dot(offset).add(DISTANCE_EPSILON));
}
