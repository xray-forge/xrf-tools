import { Nullable } from "@xrf/types";

import { DDS_FULL_BLOCK_BYTES, DDS_HALF_BLOCK_BYTES, EDdsBlockFormat } from "#/texture/dds/dds-block-format";
import { TDdsLayout, toDdsBlockLayout } from "#/texture/dds/dds-layout";

/** The four character tag a `u32` spells, low byte first. */
export function toDdsFourCc(value: number): string {
  return String.fromCharCode(value & 0xff, (value >> 8) & 0xff, (value >> 16) & 0xff, (value >> 24) & 0xff);
}

/**
 * The layout a `DDPF_FOURCC` tag names.
 *
 * @param fourCc - The four character tag, as the pixel format stores it.
 * @returns The layout, or null for a tag this does not model.
 */
export function getDdsFourCcLayout(fourCc: string): Nullable<TDdsLayout> {
  switch (fourCc) {
    case "DXT1":
      return toDdsBlockLayout(EDdsBlockFormat.BC1, DDS_HALF_BLOCK_BYTES);

    case "DXT2":
    case "DXT3":
      return toDdsBlockLayout(EDdsBlockFormat.BC2, DDS_FULL_BLOCK_BYTES);

    case "DXT4":
    case "DXT5":
      return toDdsBlockLayout(EDdsBlockFormat.BC3, DDS_FULL_BLOCK_BYTES);

    // One plane of the pair a bump map is stored as, which the sdk writes under either spelling.
    case "ATI1":
    case "BC4U":
      return toDdsBlockLayout(EDdsBlockFormat.BC4, DDS_HALF_BLOCK_BYTES);

    case "BC4S":
      return toDdsBlockLayout(EDdsBlockFormat.BC4_SIGNED, DDS_HALF_BLOCK_BYTES);

    case "ATI2":
    case "BC5U":
      return toDdsBlockLayout(EDdsBlockFormat.BC5, DDS_FULL_BLOCK_BYTES);

    case "BC5S":
      return toDdsBlockLayout(EDdsBlockFormat.BC5_SIGNED, DDS_FULL_BLOCK_BYTES);

    default:
      return null;
  }
}
