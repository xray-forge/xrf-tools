import { DepthTexture, Texture } from "three/webgpu";

/**
 * The G-buffer's attachments, as the passes after it sample them.
 */
export interface IGBufferTextures {
  /** Raw albedo in colour, gloss in alpha: byte for byte the engine's `rt_Color`. */
  readonly albedo: Texture;
  /** The octahedral view space normal. */
  readonly normal: Texture;
  /** Hemisphere occlusion, sun occlusion, lighting model slice, flags. */
  readonly surface: Texture;
  readonly depth: DepthTexture;
}
