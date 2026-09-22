import { Nullable } from "@xrf/types";

import { EDdsChannels } from "#/texture/dds/dds-channels";
import { TDdsLayout, toDdsTexelLayout } from "#/texture/dds/dds-layout";

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
 * @returns The layout, or null for a layout whose channels are not whole bytes.
 */
export function getDdsMaskLayout(masks: IDdsChannelMasks): Nullable<TDdsLayout> {
  const { bitCount, red, green, blue, alpha } = masks;

  if (bitCount === 32 && red === BYTE_2 && green === BYTE_1 && blue === BYTE_0) {
    return toDdsTexelLayout(EDdsChannels.BGRA);
  }

  if (bitCount === 32 && red === BYTE_0 && green === BYTE_1 && blue === BYTE_2 && alpha === BYTE_3) {
    return toDdsTexelLayout(EDdsChannels.RGBA);
  }

  if (bitCount === 24 && red === BYTE_2 && green === BYTE_1 && blue === BYTE_0) {
    return toDdsTexelLayout(EDdsChannels.BGR);
  }

  return null;
}

/** The masks as a refusal names them. */
export function describeDdsMasks(masks: IDdsChannelMasks): string {
  return (
    `an uncompressed ${masks.bitCount} bit layout, ` +
    `r=${toMask(masks.red)} g=${toMask(masks.green)} b=${toMask(masks.blue)} a=${toMask(masks.alpha)}`
  );
}

function toMask(mask: number): string {
  return `0x${(mask >>> 0).toString(16).padStart(8, "0")}`;
}
