import { Node } from "three/webgpu";

/**
 * A surface at one texel before any light: what the G-buffer stores, and what the forward path lights.
 */
export interface ISurfaceTexel {
  /** Raw albedo, detail and tint applied. */
  albedo: Node<"vec3">;
  /** The base's alpha, which only a cut-out or composited surface reads. */
  alpha: Node<"float">;
  /** The view space normal, bumped where the surface binds a pair. */
  normal: Node<"vec3">;
  gloss: Node<"float">;
  /** The baked hemisphere occlusion. */
  hemi: Node<"float">;
  /** The baked sun occlusion. */
  sun: Node<"float">;
  /** The lighting model slice. */
  slice: Node<"float">;
}
