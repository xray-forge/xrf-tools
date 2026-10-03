/**
 * How the finished frame's edges are smoothed. Deferred shading rules hardware multisampling out, so every mode is a
 * pass over the frame.
 */
export enum ERendererAntialiasing {
  NONE = "none",
  /** One pass, softest, cheapest. */
  FXAA = "fxaa",
  /** Three passes over the frame's edges, crisp and stable, `Base`'s choice. */
  SMAA = "smaa",
  /**
   * Temporal: every frame's samples jittered within the pixel and resolved with the frames before, found by the motion
   * every surface writes. Smooths inside a surface too, cut-out foliage and thin wires, where SMAA finds no edge.
   */
  TAA = "taa",
  /**
   * FSR 2: AMD's temporal upscaler, from the same jittered frames, motion and depth as TAA, with locks that keep thin
   * features and a reactive mask from what the blended surfaces changed.
   */
  FSR2 = "fsr2",
}
