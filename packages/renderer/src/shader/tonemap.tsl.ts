import { Node } from "three/webgpu";

/** `fWhiteIntensity` of `tonemap`, squared. */
const WHITE_INTENSITY_SQUARED: number = 1.7 * 1.7;

/**
 * `tonemap` of `common_functions.h`, the curve `toneMapReinhard` states on the CPU.
 *
 * @param color - The lit colour.
 * @param scale - What adaptation multiplies by first.
 * @returns The tonemapped colour.
 */
export function toToneMapped(color: Node<"vec3">, scale: Node<"float">): Node<"vec3"> {
  const x = color.mul(scale);

  return x.mul(x.div(WHITE_INTENSITY_SQUARED).add(1)).div(x.add(1));
}
