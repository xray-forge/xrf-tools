import { ERendererTextureEncoding, TRendererTextureSource } from "@xrf/renderer";

import { ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";
import { createRenderCheckerSource } from "@/core/render/lib/texture/render-texture-source";

/** The stand-in's side, in texels: small, since it tiles, and all that matters is that it reads as a pattern. */
const CHECKER_SIZE: number = 16;

/** Magenta and near black, the colours a missing texture has meant since before any of this. */
const CHECKER_COLORS: readonly [readonly [number, number, number], readonly [number, number, number]] = [
  [255, 0, 255],
  [16, 16, 16],
];

/**
 * @param delivery - Where one file is fetched from, or the reason it cannot be.
 * @returns What the renderer uploads: the file it fetches, falling back to the backend's picture of it, or a checker
 *   standing in.
 */
export function toLevelTextureSource(delivery: ILevelTextureDelivery): TRendererTextureSource {
  return delivery.requests
    ? { encoding: ERendererTextureEncoding.FETCH, file: delivery.requests.file, picture: delivery.requests.picture }
    : createLevelCheckerSource();
}

/**
 * A stand-in that survives whatever would hide it: opaque, since the surfaces most needing one are cut out, and
 * sampled nearest, since a filtered checker at distance is the flat grey it exists to differ from.
 *
 * @returns The checker, magenta and near black, in squares of two texels.
 */
export function createLevelCheckerSource(): TRendererTextureSource {
  return createRenderCheckerSource(CHECKER_SIZE, CHECKER_COLORS, 2);
}
