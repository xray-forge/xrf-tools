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

import { Nullable } from "@/lib/types/general";

/**
 * How the texels of one dds layout are stored.
 */
export enum EDdsLayout {
  /** Four by four blocks, uploaded to the gpu exactly as the file stores them. */
  BLOCK = "block",
  /** One texel at a time, expanded into the rgba byte order three.js uploads. */
  TEXELS = "texels",
}

/**
 * The byte order a texel-stored layout keeps its channels in.
 */
export enum EDdsChannels {
  /** What `D3DFMT_A8R8G8B8` stores, which is most of what the sdk wrote uncompressed. */
  BGRA = "bgra",
  /** `D3DFMT_R8G8B8`, three bytes a texel, opaque once expanded. */
  BGR = "bgr",
  /** Already in three's own order, so expansion is a copy. */
  RGBA = "rgba",
}

/** How one dds layout reaches the gpu. */
export type TDdsLayout =
  | { kind: EDdsLayout.BLOCK; format: CompressedPixelFormat; blockBytes: number }
  | { kind: EDdsLayout.TEXELS; channels: EDdsChannels };

/** Bytes one four by four block of the s3tc and rgtc single-plane formats occupies. */
const HALF_BLOCK: number = 8;
/** Bytes one four by four block of every other block format occupies. */
const FULL_BLOCK: number = 16;

function block(format: CompressedPixelFormat, blockBytes: number): TDdsLayout {
  return { blockBytes, format, kind: EDdsLayout.BLOCK };
}

function texels(channels: EDdsChannels): TDdsLayout {
  return { channels, kind: EDdsLayout.TEXELS };
}

/**
 * The layout a `DDPF_FOURCC` tag names.
 *
 * `DXT1` answers twice, because the same blocks mean two things: the format carries one bit of alpha and X-Ray's
 * `tfADXT1` authors it, while the vast majority of files use the transparent-black block mode with nothing meaning to
 * be transparent. Whether it is read is the surface's answer, not the file's, so it arrives as an argument.
 *
 * @param fourCc - The four character tag, as the pixel format stores it.
 * @param isAlphaRead - Whether any surface drawn with this file samples its alpha channel.
 * @returns How to upload it, or null for a tag this does not model.
 */
export function getDdsFourCcLayout(fourCc: string, isAlphaRead: boolean): Nullable<TDdsLayout> {
  switch (fourCc) {
    case "DXT1":
      return block(isAlphaRead ? RGBA_S3TC_DXT1_Format : RGB_S3TC_DXT1_Format, HALF_BLOCK);

    case "DXT2":
    case "DXT3":
      return block(RGBA_S3TC_DXT3_Format, FULL_BLOCK);

    case "DXT4":
    case "DXT5":
      return block(RGBA_S3TC_DXT5_Format, FULL_BLOCK);

    // One plane of the pair a bump map is stored as, which the sdk writes under either spelling.
    case "ATI1":
    case "BC4U":
      return block(RED_RGTC1_Format, HALF_BLOCK);

    case "BC4S":
      return block(SIGNED_RED_RGTC1_Format, HALF_BLOCK);

    case "ATI2":
    case "BC5U":
      return block(RED_GREEN_RGTC2_Format, FULL_BLOCK);

    case "BC5S":
      return block(SIGNED_RED_GREEN_RGTC2_Format, FULL_BLOCK);

    default:
      return null;
  }
}

/**
 * The layout a `DXGI_FORMAT` names, for a file carrying the `DX10` extended header.
 *
 * @param dxgiFormat - The code the extended header carries.
 * @param isAlphaRead - Whether any surface drawn with this file samples its alpha channel; only `BC1` reads it.
 * @returns How to upload it, or null for a code this does not model.
 */
export function getDdsDxgiLayout(dxgiFormat: number, isAlphaRead: boolean): Nullable<TDdsLayout> {
  switch (dxgiFormat) {
    // BC1, which is DXT1 under another name.
    case 70:
    case 71:
    case 72:
      return block(isAlphaRead ? RGBA_S3TC_DXT1_Format : RGB_S3TC_DXT1_Format, HALF_BLOCK);

    // BC2, which is DXT3.
    case 73:
    case 74:
    case 75:
      return block(RGBA_S3TC_DXT3_Format, FULL_BLOCK);

    // BC3, which is DXT5. The single commonest layout this viewer used to refuse: 3,679 files of the project's own
    // resource pack are plain DXT5 wearing a DX10 header.
    case 76:
    case 77:
    case 78:
      return block(RGBA_S3TC_DXT5_Format, FULL_BLOCK);

    // BC4, one plane.
    case 79:
    case 80:
      return block(RED_RGTC1_Format, HALF_BLOCK);

    case 81:
      return block(SIGNED_RED_RGTC1_Format, HALF_BLOCK);

    // BC5, two planes, which is how a bump pair is stored.
    case 82:
    case 83:
      return block(RED_GREEN_RGTC2_Format, FULL_BLOCK);

    case 84:
      return block(SIGNED_RED_GREEN_RGTC2_Format, FULL_BLOCK);

    // BC6H, the floating point pair.
    case 94:
    case 95:
      return block(RGB_BPTC_UNSIGNED_Format, FULL_BLOCK);

    case 96:
      return block(RGB_BPTC_SIGNED_Format, FULL_BLOCK);

    // BC7, which the reference trees ship 30 of and which three uploads natively.
    case 97:
    case 98:
    case 99:
      return block(RGBA_BPTC_Format, FULL_BLOCK);

    case 28:
    case 29:
      return texels(EDdsChannels.RGBA);

    case 87:
    case 88:
      return texels(EDdsChannels.BGRA);

    default:
      return null;
  }
}

/** Channel masks of one uncompressed pixel format, as the header stores them. */
export interface IDdsChannelMasks {
  bitCount: number;
  red: number;
  green: number;
  blue: number;
  alpha: number;
}

/** A whole byte of a thirty-two bit mask, by the place it sits in. */
const BYTE_0: number = 0x000000ff;
const BYTE_1: number = 0x0000ff00;
const BYTE_2: number = 0x00ff0000;
const BYTE_3: number = 0xff000000;

/**
 * The layout an uncompressed pixel format's channel masks name.
 *
 * Matched on the whole mask rather than on an overlap, because two of the orders differ only in which end red sits
 * at: `A8R8G8B8` and `A8B8G8R8` both set every byte, and a test that asked whether red overlapped the third byte
 * would take one for the other.
 *
 * @param masks - The pixel format's bit count and four channel masks.
 * @returns How to upload it, or null for a layout whose channels are not whole bytes.
 */
export function getDdsMaskLayout(masks: IDdsChannelMasks): Nullable<TDdsLayout> {
  const { bitCount, red, green, blue, alpha } = masks;

  if (bitCount === 32 && red === BYTE_2 && green === BYTE_1 && blue === BYTE_0) {
    return texels(EDdsChannels.BGRA);
  }

  if (bitCount === 32 && red === BYTE_0 && green === BYTE_1 && blue === BYTE_2 && alpha === BYTE_3) {
    return texels(EDdsChannels.RGBA);
  }

  if (bitCount === 24 && red === BYTE_2 && green === BYTE_1 && blue === BYTE_0) {
    return texels(EDdsChannels.BGR);
  }

  return null;
}

/** Bytes one texel of a layout occupies in the file, which is not what it occupies once expanded. */
export function getDdsSourceStride(channels: EDdsChannels): number {
  return channels === EDdsChannels.BGR ? 3 : 4;
}
