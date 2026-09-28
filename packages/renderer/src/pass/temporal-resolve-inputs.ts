import { DepthTexture, Texture } from "three/webgpu";

/** What the resolve reads: this frame at its own size, and the history at the output's. */
export interface ITemporalResolveInputs {
  /** The tonemapped frame, drawn jittered. */
  frame: Texture;
  depth: DepthTexture;
  /** How far each pixel's surface moved since the frame before, in texture coordinates, the sky's included. */
  motion: Texture;
  /** The resolved frame before: colour, and the distance along the view of what it showed. */
  history: Texture;
}
