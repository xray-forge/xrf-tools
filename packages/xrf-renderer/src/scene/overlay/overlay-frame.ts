import { PerspectiveCamera } from "three/webgpu";

/**
 * What an overlay follows from the frame about to be drawn.
 */
export interface IOverlayFrame {
  /** The drawing camera, with its matrices current. */
  camera: PerspectiveCamera;
  /** One device pixel's size at depth one, which a sprite without attenuation is scaled by. */
  pixel: number;
}
