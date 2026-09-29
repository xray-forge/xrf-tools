import { Discard, dot, float, Fn, fract, fwidth, If, max, saturate, screenCoordinate, select, vec2 } from "three/tsl";
import { Node } from "three/webgpu";

import { IHashedAlphaCut } from "#/shader/hashed-alpha-cut";

/** `def_aref`: where a cut-out surface without its own reference is cut. */
export const DEFAULT_ALPHA_REFERENCE: number = 200 / 255;

/**
 * `clip(alpha - aref)`: an output that discards the texel first where the alpha is at or below the reference.
 *
 * @param alpha - The alpha the surface is cut by.
 * @param output - What it writes where it stands.
 * @param reference - Where it is cut.
 * @returns The output, cut.
 */
export function toAlphaCut<T extends "vec4" | "float">(
  alpha: Node<"float">,
  output: Node<T>,
  reference: Node<"float">
): Node<T> {
  return Fn(() => {
    If(alpha.lessThanEqual(reference), () => {
      Discard();
    });

    return output;
  })() as unknown as Node<T>;
}

/**
 * `clip(alpha - aref)` as a temporal resolve can average: the alpha sharpened to a pixel wide ramp about the reference,
 * cut against a threshold that moves from pixel to pixel and frame to frame. Near, the ramp is the edge; far, where a
 * pixel spans leaves and gaps, it is the share of the pixel covered, which the resolve turns into a soft canopy rather
 * than specks. Without a temporal resolve it cuts at the reference as `toAlphaCut` does.
 *
 * @param cut - The alpha, the output, the reference and the settings.
 * @returns The output, cut.
 */
export function toHashedAlphaCut<T extends "vec4" | "float">(cut: IHashedAlphaCut<T>): Node<T> {
  const { alpha, output, reference, settings } = cut;

  return Fn(() => {
    const ramp: Node<"float"> = saturate(
      alpha
        .sub(reference)
        .div(max(fwidth(alpha), float(1 / 255)))
        .add(0.5)
    );
    // Interleaved gradient noise, turned each frame: a threshold a resolve averages evenly.
    const seed: Node<"vec2"> = screenCoordinate.xy.add(settings.frame.mul(5.588238));
    const noise: Node<"float"> = fract(fract(dot(seed, vec2(0.06711056, 0.00583715))).mul(52.9829189));
    const threshold: Node<"float"> = select(settings.stochastic.greaterThan(0.5), noise, float(0.5));

    If(ramp.lessThanEqual(threshold), () => {
      Discard();
    });

    return output;
  })() as unknown as Node<T>;
}
