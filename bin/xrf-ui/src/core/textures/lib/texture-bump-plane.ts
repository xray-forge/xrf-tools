import { IDdsTexels } from "@xrf/dds";

import { ITextureBumpTexels } from "@/core/textures/lib/texture-surface";

/** One plane of a bump pair, as the channels panel shows it. */
export enum ETextureBumpPlane {
  /** `normal.gloss` as stored: the normal reversed into green, blue and alpha, gloss in red. */
  BUMP = "bump",
  /** `normal_error.height` as stored: the quantisation error of the normal in rgb, height in alpha. */
  COMPANION = "companion",
  /** The tangent space normal the engine reconstructs, mapped into the unit range. */
  NORMAL = "normal",
  /** Gloss, the bump's red squared. */
  GLOSS = "gloss",
  /** Height as the companion stores it in alpha. */
  HEIGHT = "height",
}

/**
 * One plane of a bump pair as opaque rgba bytes, a texel a pixel at the bump's own size, through the decode the engine
 * shades with (`decodeXrayBumpTexel`, written out here since a plane runs it for every texel of the file).
 *
 * @param texels - Both halves on the cpu.
 * @param plane - The plane wanted.
 * @returns Its pixels, row major at the bump's width, the row X-Ray stores first coming first.
 */
export function toTextureBumpPlane(texels: ITextureBumpTexels, plane: ETextureBumpPlane): Uint8ClampedArray {
  const { bump, companion } = texels;
  const { width, height } = bump;
  const pixels: Uint8ClampedArray = new Uint8ClampedArray(width * height * 4);

  for (let y: number = 0; y < height; y += 1) {
    // A companion of another size is read at the same place across it, as a sampler would.
    const companionRow: number = Math.min(companion.height - 1, Math.floor((y * companion.height) / height));

    for (let x: number = 0; x < width; x += 1) {
      const at: number = (y * width + x) * 4;
      const companionAt: number =
        (companionRow * companion.width + Math.min(companion.width - 1, Math.floor((x * companion.width) / width))) * 4;

      writePlaneTexel(pixels, at, plane, bump, at, companion, companionAt);
    }
  }

  return pixels;
}

function writePlaneTexel(
  pixels: Uint8ClampedArray,
  at: number,
  plane: ETextureBumpPlane,
  bump: IDdsTexels,
  bumpAt: number,
  companion: IDdsTexels,
  companionAt: number
): void {
  const nu: Uint8Array = bump.data;
  const nuE: Uint8Array = companion.data;

  pixels[at + 3] = 255;

  switch (plane) {
    case ETextureBumpPlane.BUMP:
      pixels.set(nu.subarray(bumpAt, bumpAt + 3), at);

      return;

    case ETextureBumpPlane.COMPANION:
      pixels.set(nuE.subarray(companionAt, companionAt + 3), at);

      return;

    case ETextureBumpPlane.NORMAL:
      // `Nu.wzy + (NuE.xyz - 1)`, then from the signed range into the unit one.
      for (let axis: number = 0; axis < 3; axis += 1) {
        const normal: number = (nu[bumpAt + 3 - axis] + nuE[companionAt + axis]) / 255 - 1;

        pixels[at + axis] = (normal * 0.5 + 0.5) * 255;
      }

      return;

    case ETextureBumpPlane.GLOSS:
      pixels.fill((nu[bumpAt] * nu[bumpAt]) / 255, at, at + 3);

      return;

    case ETextureBumpPlane.HEIGHT:
      pixels.fill(nuE[companionAt + 3], at, at + 3);

      return;
  }
}
