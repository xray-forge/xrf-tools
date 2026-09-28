/**
 * Where one frame's samples stand within their pixels, and the cycle they take their places from.
 */
export interface IRendererFrameJitter {
  /** The offset, in drawn pixels about each pixel's centre, `y` down: the texel `m` shows `m + 0.5 + offset`. */
  readonly offset: readonly [number, number];
  /** How many places the cycle holds. */
  readonly phases: number;
}
