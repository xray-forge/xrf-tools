import { dot, fract, vec2, vec3 } from "three/tsl";
import { Node } from "three/webgpu";

/** One step of the eight-bit canvas the frame is written to. */
const OUTPUT_STEP: number = 1 / 255;

/**
 * Half a step of the canvas either way, by interleaved gradient noise (Jimenez, 2014): a flat gradient, the haze under
 * a sky or fog thinning out, then falls between two steps as a fine grain rather than as bands a step apart.
 *
 * @param pixel - The pixel's coordinate on the canvas.
 * @returns What to add to its colour before it is written.
 */
export function toOutputDither(pixel: Node<"vec2">): Node<"vec3"> {
  const noise: Node<"float"> = fract(fract(dot(pixel, vec2(0.06711056, 0.00583715))).mul(52.9829189));

  return vec3(noise.sub(0.5).mul(OUTPUT_STEP));
}
