import { uniform } from "three/tsl";

/**
 * @param sharpening - How sharp, from none to the most.
 * @returns RCAS's strength: FSR 2's remap of a sharpness to stops, `(1 - sharpening) * 2`, as `exp2(-stops)`.
 */
export function toSharpenStrength(sharpening: number): number {
  return 2 ** -((1 - Math.min(Math.max(sharpening, 0), 1)) * 2);
}

/**
 * What the sharpening of an upscaled frame reads.
 */
export class SharpenUniforms {
  /** RCAS's `con`: `exp2(-stops)`, one sharpening most. */
  public readonly strength = uniform(1);

  /**
   * @param sharpening - How sharp, from none to the most.
   */
  public apply(sharpening: number): void {
    this.strength.value = toSharpenStrength(sharpening);
  }
}
