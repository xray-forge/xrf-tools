import { ITextureBumpTexels } from "@/core/textures/lib/texture-surface";
import { ITextureTexels } from "@/core/textures/lib/texture-texels";

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
  // A companion of another size is read at the same place across it, as a sampler would: each column's byte offset
  // into a companion row, worked out once for every row.
  const columns: Uint32Array = new Uint32Array(width);

  for (let x: number = 0; x < width; x += 1) {
    columns[x] = Math.min(companion.width - 1, Math.floor((x * companion.width) / width)) * 4;
  }

  for (let y: number = 0; y < height; y += 1) {
    const companionRow: number =
      Math.min(companion.height - 1, Math.floor((y * companion.height) / height)) * companion.width * 4;

    for (let x: number = 0; x < width; x += 1) {
      const at: number = (y * width + x) * 4;

      writePlaneTexel(pixels, at, plane, bump, at, companion, companionRow + columns[x]);
    }
  }

  return pixels;
}

function writePlaneTexel(
  pixels: Uint8ClampedArray,
  at: number,
  plane: ETextureBumpPlane,
  bump: ITextureTexels,
  bumpAt: number,
  companion: ITextureTexels,
  companionAt: number
): void {
  const nu: Uint8Array = bump.data;
  const nuE: Uint8Array = companion.data;

  // Written byte by byte: a view or a fill per texel costs more than the texel's whole decode.
  pixels[at + 3] = 255;

  switch (plane) {
    case ETextureBumpPlane.BUMP:
      pixels[at] = nu[bumpAt];
      pixels[at + 1] = nu[bumpAt + 1];
      pixels[at + 2] = nu[bumpAt + 2];

      return;

    case ETextureBumpPlane.COMPANION:
      pixels[at] = nuE[companionAt];
      pixels[at + 1] = nuE[companionAt + 1];
      pixels[at + 2] = nuE[companionAt + 2];

      return;

    case ETextureBumpPlane.NORMAL:
      // `Nu.wzy + (NuE.xyz - 1)`, then from the signed range into the unit one.
      for (let axis: number = 0; axis < 3; axis += 1) {
        const normal: number = (nu[bumpAt + 3 - axis] + nuE[companionAt + axis]) / 255 - 1;

        pixels[at + axis] = (normal * 0.5 + 0.5) * 255;
      }

      return;

    case ETextureBumpPlane.GLOSS: {
      const gloss: number = (nu[bumpAt] * nu[bumpAt]) / 255;

      pixels[at] = gloss;
      pixels[at + 1] = gloss;
      pixels[at + 2] = gloss;

      return;
    }

    case ETextureBumpPlane.HEIGHT: {
      const stored: number = nuE[companionAt + 3];

      pixels[at] = stored;
      pixels[at + 1] = stored;
      pixels[at + 2] = stored;

      return;
    }
  }
}
