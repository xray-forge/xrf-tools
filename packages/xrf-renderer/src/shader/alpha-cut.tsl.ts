import { Discard, Fn, If } from "three/tsl";
import { Node } from "three/webgpu";

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
