import { DDS_BLOCK_SIZE, EDdsBlockFormat } from "#/texture/dds/dds-block-format";
import { EDdsChannels, getDdsSourceStride } from "#/texture/dds/dds-channels";

/**
 * How the texels of one dds layout are stored.
 */
export enum EDdsLayout {
  /** Four by four blocks, uploaded to the gpu exactly as the file stores them. */
  BLOCK = "block",
  /** One texel at a time, expanded into rgba byte order. */
  TEXELS = "texels",
}

/** How one dds layout is stored. */
export type TDdsLayout =
  | { kind: EDdsLayout.BLOCK; format: EDdsBlockFormat; blockBytes: number }
  | { kind: EDdsLayout.TEXELS; channels: EDdsChannels };

/** A block layout. */
export function toDdsBlockLayout(format: EDdsBlockFormat, blockBytes: number): TDdsLayout {
  return { blockBytes, format, kind: EDdsLayout.BLOCK };
}

/** A texel layout. */
export function toDdsTexelLayout(channels: EDdsChannels): TDdsLayout {
  return { channels, kind: EDdsLayout.TEXELS };
}

/** Bytes one mip occupies **in the file**, which for a texel layout is not what it occupies once expanded. */
export function getDdsStoredLength(layout: TDdsLayout, width: number, height: number): number {
  if (layout.kind === EDdsLayout.BLOCK) {
    // A chain runs below one block, and the smallest levels still cost a whole one.
    return (
      (Math.max(DDS_BLOCK_SIZE, width) / DDS_BLOCK_SIZE) *
      (Math.max(DDS_BLOCK_SIZE, height) / DDS_BLOCK_SIZE) *
      layout.blockBytes
    );
  }

  return width * height * getDdsSourceStride(layout.channels);
}
