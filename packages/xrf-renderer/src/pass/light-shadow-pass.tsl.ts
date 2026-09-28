import { float, floor, ivec2, screenCoordinate, textureLoad, vec4 } from "three/tsl";
import { DepthTexture, Node } from "three/webgpu";

/** Reversed depth's far plane, which a square of the atlas is cleared to before its face is drawn. */
export function toFarDepth(): Node<"float"> {
  return float(0);
}

/** What a depth-only draw writes to the colour it cannot go without. */
export function toNoColor(): Node<"vec4"> {
  return vec4(0);
}

/**
 * @param still - The atlas of what stands still, each face's square as it was last drawn in full.
 * @returns Its depth at the texel drawn: what a face drawn again over what sways starts from.
 */
export function toKeptDepth(still: DepthTexture): Node<"float"> {
  return textureLoad(still, ivec2(floor(screenCoordinate.xy))).x as unknown as Node<"float">;
}
