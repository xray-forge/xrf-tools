/**
 * How a surface reaches the frame, as the engine's blender for it decides.
 */
export enum ERendererDraw {
  /** Into the G-buffer, every texel. */
  OPAQUE = "opaque",
  /** Into the G-buffer, texels at or below the reference discarded. */
  CUT_OUT = "cutOut",
  /** Composited by alpha, after the deferred passes. */
  BLENDED = "blended",
  /** Added to what is under it, whole. */
  ADDED = "added",
  /** Added to what is under it, weighted by its own alpha. */
  ALPHA_ADDED = "alphaAdded",
  /** Multiplied into what is under it. */
  MULTIPLIED = "multiplied",
  /** Multiplied at twice the strength, so mid grey leaves what is under it alone. */
  MULTIPLIED_2X = "multiplied2x",
  /** Submitted and drawn, writing nothing. */
  INVISIBLE = "invisible",
  /** Water, as the engine's `water` programs draw it: rippled, reflecting the sky, distorting what is behind it. */
  WATER = "water",
}
