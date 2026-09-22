import { Fn } from "three/tsl";
import { Node } from "three/webgpu";

/** `fWhiteIntensity` of `tonemap`, squared. */
const WHITE_INTENSITY_SQUARED: number = 1.7 * 1.7;

/**
 * `tonemap` of `common_functions.h`, the curve `toneMapReinhard` states on the CPU.
 */
export const toneMapReinhardNode = Fn(([color, scale]: [Node<"vec3">, Node<"float">]) => {
  const x = color.mul(scale);

  return x.mul(x.div(WHITE_INTENSITY_SQUARED).add(1)).div(x.add(1));
});
