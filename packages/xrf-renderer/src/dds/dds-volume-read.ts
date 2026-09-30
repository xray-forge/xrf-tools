import { Nullable } from "@xrf/types";

import { DDS_BLOCK_SIZE, EDdsBlockFormat } from "#/dds/dds-block-format";
import { getDdsFourCcLayout } from "#/dds/dds-fourcc";
import { IDdsHeader, readDdsHeader } from "#/dds/dds-header";
import { EDdsLayout, TDdsLayout } from "#/dds/dds-layout";
import { IDdsVolume } from "#/dds/dds-volume";

/** The block formats a volume is decoded from: DXT1, DXT3 and DXT5. */
const DECODED: ReadonlySet<EDdsBlockFormat> = new Set([EDdsBlockFormat.BC1, EDdsBlockFormat.BC2, EDdsBlockFormat.BC3]);

/** Bytes a decoded block takes: sixteen texels of four. */
const DECODED_BLOCK_BYTES: number = DDS_BLOCK_SIZE * DDS_BLOCK_SIZE * 4;

/**
 * A volume the texture reader refuses, since no surface draws one, read as `rain_patch_normal` samples it: its top
 * level, its blocks decoded on the CPU, because a GPU samples a compressed volume only with a feature it may lack.
 *
 * @param bytes - The file as stored.
 * @returns The volume, or null for a file that is no DXT1, DXT3 or DXT5 volume.
 */
export function readDdsVolume(bytes: ArrayBuffer): Nullable<IDdsVolume> {
  const header: Nullable<IDdsHeader> = readDdsHeader(bytes).header;
  const layout: Nullable<TDdsLayout> = header?.volume ? getDdsFourCcLayout(header.fourCc) : null;

  if (!header?.volume || layout?.kind !== EDdsLayout.BLOCK || !DECODED.has(layout.format)) {
    return null;
  }

  const { width, height, dataOffset } = header;
  const { depth } = header.volume;
  const { format, blockBytes } = layout;
  const across: number = Math.ceil(width / DDS_BLOCK_SIZE);
  const down: number = Math.ceil(height / DDS_BLOCK_SIZE);

  if (dataOffset + across * down * depth * blockBytes > bytes.byteLength) {
    return null;
  }

  const source: Uint8Array = new Uint8Array(bytes, dataOffset);
  const rgba: Uint8Array = new Uint8Array(width * height * depth * 4);
  const block: Uint8Array = new Uint8Array(DECODED_BLOCK_BYTES);

  for (let slice: number = 0; slice < depth; slice += 1) {
    for (let row: number = 0; row < down; row += 1) {
      for (let column: number = 0; column < across; column += 1) {
        const at: number = ((slice * down + row) * across + column) * blockBytes;

        decodeBlock(source, at, format, block);

        for (let y: number = 0; y < DDS_BLOCK_SIZE && row * DDS_BLOCK_SIZE + y < height; y += 1) {
          for (let x: number = 0; x < DDS_BLOCK_SIZE && column * DDS_BLOCK_SIZE + x < width; x += 1) {
            const texel: number = (y * DDS_BLOCK_SIZE + x) * 4;
            const to: number = ((slice * height + row * DDS_BLOCK_SIZE + y) * width + column * DDS_BLOCK_SIZE + x) * 4;

            rgba.set(block.subarray(texel, texel + 4), to);
          }
        }
      }
    }
  }

  return { depth, height, rgba, width };
}

/** One block's sixteen texels, four bytes each, row by row. */
function decodeBlock(source: Uint8Array, at: number, format: EDdsBlockFormat, out: Uint8Array): void {
  const isBc1: boolean = format === EDdsBlockFormat.BC1;

  // Every format but DXT1 stores its alpha first, in the block's first half.
  decodeColors(source, isBc1 ? at : at + 8, isBc1, out);

  if (format === EDdsBlockFormat.BC3) {
    decodeInterpolatedAlpha(source, at, out);
  } else if (format === EDdsBlockFormat.BC2) {
    for (let texel: number = 0; texel < 16; texel += 1) {
      const nibble: number = (source[at + (texel >> 1)] >> ((texel & 1) * 4)) & 0xf;

      out[texel * 4 + 3] = nibble * 17;
    }
  }
}

/** The colour half of a block: two 565 end points and a two-bit index a texel. */
function decodeColors(source: Uint8Array, at: number, isPunched: boolean, out: Uint8Array): void {
  const first: number = source[at] | (source[at + 1] << 8);
  const second: number = source[at + 2] | (source[at + 3] << 8);
  const a: ReadonlyArray<number> = toRgb(first);
  const b: ReadonlyArray<number> = toRgb(second);
  const isFour: boolean = !isPunched || first > second;
  const palette: ReadonlyArray<ReadonlyArray<number>> = isFour
    ? [a, b, a.map((it, i) => (2 * it + b[i]) / 3), a.map((it, i) => (it + 2 * b[i]) / 3)]
    : [a, b, a.map((it, i) => (it + b[i]) / 2), [0, 0, 0]];
  const indices: number =
    (source[at + 4] | (source[at + 5] << 8) | (source[at + 6] << 16) | (source[at + 7] << 24)) >>> 0;

  for (let texel: number = 0; texel < 16; texel += 1) {
    const index: number = (indices >>> (texel * 2)) & 3;
    const color: ReadonlyArray<number> = palette[index];

    out[texel * 4] = Math.round(color[0]);
    out[texel * 4 + 1] = Math.round(color[1]);
    out[texel * 4 + 2] = Math.round(color[2]);
    out[texel * 4 + 3] = !isFour && index === 3 ? 0 : 255;
  }
}

/** DXT5's alpha half: two end points and a three-bit index a texel. */
function decodeInterpolatedAlpha(source: Uint8Array, at: number, out: Uint8Array): void {
  const first: number = source[at];
  const second: number = source[at + 1];
  const palette: Array<number> = [first, second];

  if (first > second) {
    for (let step: number = 1; step < 7; step += 1) {
      palette.push(((7 - step) * first + step * second) / 7);
    }
  } else {
    for (let step: number = 1; step < 5; step += 1) {
      palette.push(((5 - step) * first + step * second) / 5);
    }

    palette.push(0, 255);
  }

  for (let texel: number = 0; texel < 16; texel += 1) {
    const bit: number = texel * 3;
    const byte: number = at + 2 + (bit >> 3);
    const word: number = source[byte] | (source[byte + 1] << 8);
    const index: number = (word >> (bit & 7)) & 7;

    out[texel * 4 + 3] = Math.round(palette[index]);
  }
}

/** A 565 colour's channels in `[0, 255]`. */
function toRgb(color: number): ReadonlyArray<number> {
  return [((color >> 11) & 31) * (255 / 31), ((color >> 5) & 63) * (255 / 63), (color & 31) * (255 / 31)];
}
