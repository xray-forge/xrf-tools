import { round, screenUV } from "three/tsl";
import { DepthTexture, Node, Texture } from "three/webgpu";

import { loadClamped, loadDepth, toTextureSize } from "#/shader/texel.tsl";

/**
 * @param size - The drawing's size.
 * @param at - A point, in texture coordinates of the output.
 * @param jitter - This frame's jitter, in drawn pixels: the sample at texel `m` stands at `m + 0.5 + jitter`.
 * @returns The drawn texel whose sample stands nearest the point, held as floats.
 */
export function toNearestDrawnTexel(size: Node<"vec2">, at: Node<"vec2">, jitter: Node<"vec2">): Node<"vec2"> {
  return round(at.mul(size).sub(0.5).sub(jitter));
}

/**
 * @param frame - The frame as drawn, whose size the depth shares.
 * @param depth - The drawn depth.
 * @param jitter - This frame's jitter, in drawn pixels.
 * @returns The depth an upscaled output carries for the helpers: the drawn sample's nearest each output pixel's centre.
 */
export function toUpscaledDepth(frame: Texture, depth: DepthTexture, jitter: Node<"vec2">): Node<"float"> {
  const size: Node<"vec2"> = toTextureSize(frame);

  return loadDepth(depth, toNearestDrawnTexel(size, screenUV, jitter), size);
}

/**
 * @param frame - The frame as drawn.
 * @param jitter - This frame's jitter, in drawn pixels.
 * @returns Its coverage at the drawn sample nearest each output pixel's centre, which present shows as alpha.
 */
export function toUpscaledCoverage(frame: Texture, jitter: Node<"vec2">): Node<"float"> {
  const size: Node<"vec2"> = toTextureSize(frame);

  return loadClamped(frame, toNearestDrawnTexel(size, screenUV, jitter), size).w;
}
