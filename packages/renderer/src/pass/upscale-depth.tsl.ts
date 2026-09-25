import { clamp, int, ivec2, round, screenUV, texture, textureLoad, vec2 } from "three/tsl";
import { DepthTexture, Node, Texture } from "three/webgpu";

/**
 * @param frame - The frame as drawn, whose size the depth shares.
 * @param at - A point, in texture coordinates of the output.
 * @param jitter - This frame's jitter, in drawn pixels: the sample at texel `m` stands at `m + 0.5 + jitter`.
 * @returns The drawn texel whose sample stands nearest the point.
 */
export function toNearestDrawnTexel(frame: Texture, at: Node<"vec2">, jitter: Node<"vec2">): Node<"ivec2"> {
  const size: Node<"vec2"> = vec2(texture(frame).size(int(0)) as Node<"uvec2">);

  return ivec2(clamp(round(at.mul(size).sub(0.5).sub(jitter)), vec2(0), size.sub(1)));
}

/**
 * The depth an upscaled output carries for the helpers drawn over it: the drawn sample's nearest each output pixel's
 * centre, reversed like the drawn depth.
 *
 * @param frame - The frame as drawn, whose size the depth shares.
 * @param depth - The drawn depth.
 * @param jitter - This frame's jitter, in drawn pixels.
 * @returns The depth.
 */
export function toUpscaledDepth(frame: Texture, depth: DepthTexture, jitter: Node<"vec2">): Node<"float"> {
  return textureLoad(depth, toNearestDrawnTexel(frame, screenUV, jitter)) as unknown as Node<"float">;
}

/**
 * @param frame - The frame as drawn.
 * @param jitter - This frame's jitter, in drawn pixels.
 * @returns Its coverage at the drawn sample nearest each output pixel's centre, which present shows as alpha.
 */
export function toUpscaledCoverage(frame: Texture, jitter: Node<"vec2">): Node<"float"> {
  return textureLoad(frame, toNearestDrawnTexel(frame, screenUV, jitter)).w;
}
