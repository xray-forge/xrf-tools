import { dot } from "three/tsl";
import { Node } from "three/webgpu";

import { GrassUniforms } from "#/uniforms/grass-uniforms";

/** `EPS_L`, which every slot's box grows by. */
export const GRASS_BOX_GROWTH: number = 0.001;

/**
 * @param uniforms - The view's planes among them.
 * @param center - A sphere's centre, in renderer space.
 * @param radius - Its radius.
 * @returns Whether it lies wholly outside one of the view's planes.
 */
export function isGrassOutsideView(uniforms: GrassUniforms, center: Node<"vec3">, radius: Node<"float">): Node<"bool"> {
  const first: Node<"vec4"> = uniforms.planeNodes.element(0) as unknown as Node<"vec4">;
  let outside: Node<"bool"> = dot(first.xyz, center).add(first.w).lessThan(radius.negate());

  for (let plane = 1; plane < 6; plane++) {
    const equation: Node<"vec4"> = uniforms.planeNodes.element(plane) as unknown as Node<"vec4">;

    outside = outside.or(dot(equation.xyz, center).add(equation.w).lessThan(radius.negate()));
  }

  return outside;
}
