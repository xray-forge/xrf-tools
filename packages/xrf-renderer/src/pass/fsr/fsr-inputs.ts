import { DepthTexture, Texture } from "three/webgpu";

/** What FSR 2 reads of the frame as drawn. */
export interface IFsrInputs {
  /** The tonemapped frame, jittered, with the blended surfaces. */
  color: Texture;
  depth: DepthTexture;
  /** The renderer's motion: how far a pixel's surface moved since the frame before, in texture coordinates. */
  motion: Texture;
}
