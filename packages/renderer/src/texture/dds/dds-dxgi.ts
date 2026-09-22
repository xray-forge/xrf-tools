import { Nullable } from "@xrf/types";

import { DDS_FULL_BLOCK_BYTES, DDS_HALF_BLOCK_BYTES, EDdsBlockFormat } from "#/texture/dds/dds-block-format";
import { EDdsChannels } from "#/texture/dds/dds-channels";
import { TDdsLayout, toDdsBlockLayout, toDdsTexelLayout } from "#/texture/dds/dds-layout";

/**
 * The layout a `DXGI_FORMAT` names, for a file carrying the `DX10` extended header.
 *
 * @param dxgiFormat - The code the extended header carries.
 * @returns The layout, or null for a code this does not model.
 */
export function getDdsDxgiLayout(dxgiFormat: number): Nullable<TDdsLayout> {
  switch (dxgiFormat) {
    case 70:
    case 71:
    case 72:
      return toDdsBlockLayout(EDdsBlockFormat.BC1, DDS_HALF_BLOCK_BYTES);

    case 73:
    case 74:
    case 75:
      return toDdsBlockLayout(EDdsBlockFormat.BC2, DDS_FULL_BLOCK_BYTES);

    case 76:
    case 77:
    case 78:
      return toDdsBlockLayout(EDdsBlockFormat.BC3, DDS_FULL_BLOCK_BYTES);

    case 79:
    case 80:
      return toDdsBlockLayout(EDdsBlockFormat.BC4, DDS_HALF_BLOCK_BYTES);

    case 81:
      return toDdsBlockLayout(EDdsBlockFormat.BC4_SIGNED, DDS_HALF_BLOCK_BYTES);

    case 82:
    case 83:
      return toDdsBlockLayout(EDdsBlockFormat.BC5, DDS_FULL_BLOCK_BYTES);

    case 84:
      return toDdsBlockLayout(EDdsBlockFormat.BC5_SIGNED, DDS_FULL_BLOCK_BYTES);

    case 94:
    case 95:
      return toDdsBlockLayout(EDdsBlockFormat.BC6H, DDS_FULL_BLOCK_BYTES);

    case 96:
      return toDdsBlockLayout(EDdsBlockFormat.BC6H_SIGNED, DDS_FULL_BLOCK_BYTES);

    // BC7, which the reference trees ship 30 of.
    case 97:
    case 98:
    case 99:
      return toDdsBlockLayout(EDdsBlockFormat.BC7, DDS_FULL_BLOCK_BYTES);

    case 28:
    case 29:
      return toDdsTexelLayout(EDdsChannels.RGBA);

    case 87:
    case 88:
      return toDdsTexelLayout(EDdsChannels.BGRA);

    default:
      return null;
  }
}
