import { float, sqrt } from "three/tsl";
import { Node } from "three/webgpu";

/** `fWhiteIntensity` of `tonemap`, squared. */
export const WHITE_INTENSITY_SQUARED: number = 1.7 * 1.7;

/**
 * `tonemap` of `common_functions.h`: the engine's Reinhard curve.
 *
 * @param color - The lit colour.
 * @param scale - What adaptation multiplies by first.
 * @returns The tonemapped colour.
 */
export function toToneMapped(color: Node<"vec3">, scale: Node<"float">): Node<"vec3"> {
  const x: Node<"vec3"> = color.mul(scale);

  return x.mul(x.div(WHITE_INTENSITY_SQUARED).add(1)).div(x.add(1));
}

/**
 * The lit colour `toToneMapped` takes to a tonemapped one: the curve's root, `x = W²/2 (sqrt((1 - y)² + 4y/W²) - (1 - y))`,
 * over the scale.
 *
 * @param color - A tonemapped colour.
 * @param scale - What adaptation multiplies by first.
 * @returns The lit colour.
 */
export function toUntoneMapped(color: Node<"vec3">, scale: Node<"float">): Node<"vec3"> {
  const rest: Node<"vec3"> = float(1).sub(color);
  const x: Node<"vec3"> = sqrt(rest.mul(rest).add(color.mul(4 / WHITE_INTENSITY_SQUARED)))
    .sub(rest)
    .mul(WHITE_INTENSITY_SQUARED / 2);

  return x.div(scale);
}
