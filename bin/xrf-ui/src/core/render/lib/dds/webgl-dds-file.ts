import { EDdsBlockFormat, EDdsLayout, IDdsFile, IDdsRead, IDdsRefusal, readDdsFile } from "@xrf/renderer";
import { Nullable } from "@xrf/types";
import {
  CompressedPixelFormat,
  RED_GREEN_RGTC2_Format,
  RED_RGTC1_Format,
  RGB_BPTC_SIGNED_Format,
  RGB_BPTC_UNSIGNED_Format,
  RGB_S3TC_DXT1_Format,
  RGBA_BPTC_Format,
  RGBA_S3TC_DXT1_Format,
  RGBA_S3TC_DXT3_Format,
  RGBA_S3TC_DXT5_Format,
  SIGNED_RED_GREEN_RGTC2_Format,
  SIGNED_RED_RGTC1_Format,
} from "three";

/** A dds file as the WebGL stack uploads it: the file, and the format three's WebGL renderer takes it as. */
export interface IWebGlDdsFile extends IDdsFile {
  /** The compressed format for a block layout; null for texels, which upload as `RGBAFormat`. */
  format: Nullable<CompressedPixelFormat>;
}

/** What a read came to for the WebGL stack: exactly one of the two is present. */
export interface IWebGlDdsRead {
  file: Nullable<IWebGlDdsFile>;
  refusal: Nullable<IDdsRefusal>;
}

/**
 * Reads a dds file for the WebGL stack, which the renderer package replaces.
 *
 * @param bytes - The file as read.
 * @param isAlphaRead - Whether any surface drawn with this file samples its alpha channel, which is the only thing
 *   that decides how `DXT1`'s identical blocks are uploaded to WebGL.
 * @returns The file with its WebGL format, or the reason WebGL cannot take it.
 */
export function readWebGlDdsFile(bytes: ArrayBuffer, isAlphaRead: boolean = false): IWebGlDdsRead {
  const read: IDdsRead = readDdsFile(bytes);

  if (!read.file) {
    return { file: null, refusal: read.refusal };
  }

  const { layout } = read.file;

  return {
    file: { ...read.file, format: layout.kind === EDdsLayout.BLOCK ? toWebGlFormat(layout.format, isAlphaRead) : null },
    refusal: null,
  };
}

/**
 * The format three's WebGL renderer uploads a block layout as.
 *
 * `DXT1` answers twice, because the same blocks mean two things: the format carries one bit of alpha and X-Ray's
 * `tfADXT1` authors it, while the vast majority of files use the transparent-black block mode with nothing meaning to
 * be transparent. WebGL fixes which reading applies at upload, so the surface's answer arrives as an argument.
 */
function toWebGlFormat(format: EDdsBlockFormat, isAlphaRead: boolean): CompressedPixelFormat {
  switch (format) {
    case EDdsBlockFormat.BC1:
      return isAlphaRead ? RGBA_S3TC_DXT1_Format : RGB_S3TC_DXT1_Format;

    case EDdsBlockFormat.BC2:
      return RGBA_S3TC_DXT3_Format;

    case EDdsBlockFormat.BC3:
      return RGBA_S3TC_DXT5_Format;

    case EDdsBlockFormat.BC4:
      return RED_RGTC1_Format;

    case EDdsBlockFormat.BC4_SIGNED:
      return SIGNED_RED_RGTC1_Format;

    case EDdsBlockFormat.BC5:
      return RED_GREEN_RGTC2_Format;

    case EDdsBlockFormat.BC5_SIGNED:
      return SIGNED_RED_GREEN_RGTC2_Format;

    case EDdsBlockFormat.BC6H:
      return RGB_BPTC_UNSIGNED_Format;

    case EDdsBlockFormat.BC6H_SIGNED:
      return RGB_BPTC_SIGNED_Format;

    case EDdsBlockFormat.BC7:
      return RGBA_BPTC_Format;
  }
}
