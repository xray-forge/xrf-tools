import { ERendererTextureEncoding, TRendererTextureSource } from "@xrf/renderer";

import { ILevelTextureDelivery } from "@/core/level/lib/render/level-render-protocol";

/** The stand-in's side, in texels: small, since it tiles, and all that matters is that it reads as a pattern. */
const CHECKER_SIZE: number = 16;

/** Magenta and near black, the colours a missing texture has meant since before any of this. */
const CHECKER_COLORS: ReadonlyArray<readonly [number, number, number]> = [
  [255, 0, 255],
  [16, 16, 16],
];

/**
 * @param delivery - One file the loader read, or the reason it could not.
 * @returns What the renderer uploads: the file, the backend's picture of it, or a checker standing in.
 */
export function toLevelTextureSource(delivery: ILevelTextureDelivery): TRendererTextureSource {
  if (delivery.reason) {
    return createLevelCheckerSource();
  }

  return delivery.isDecoded
    ? { bytes: delivery.bytes, encoding: ERendererTextureEncoding.IMAGE, type: "image/png" }
    : { bytes: delivery.bytes, encoding: ERendererTextureEncoding.DDS };
}

/**
 * A stand-in that survives whatever would hide it: opaque, since the surfaces most needing one are cut out, and
 * sampled nearest, since a filtered checker at distance is the flat grey it exists to differ from.
 *
 * @returns The checker, magenta and near black.
 */
export function createLevelCheckerSource(): TRendererTextureSource {
  const bytes: Uint8Array<ArrayBuffer> = new Uint8Array(CHECKER_SIZE * CHECKER_SIZE * 4);

  for (let y = 0; y < CHECKER_SIZE; y += 1) {
    for (let x = 0; x < CHECKER_SIZE; x += 1) {
      bytes.set([...CHECKER_COLORS[((x >> 1) + (y >> 1)) % 2], 255], (y * CHECKER_SIZE + x) * 4);
    }
  }

  return {
    bytes: bytes.buffer,
    encoding: ERendererTextureEncoding.RGBA,
    height: CHECKER_SIZE,
    isNearest: true,
    width: CHECKER_SIZE,
  };
}
