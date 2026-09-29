import { Nullable } from "@xrf/types";

import { IDdsVolume } from "#/dds/dds-volume";

/** `DDS `, little endian. */
const MAGIC: number = 0x20534444;

/** `DDSCAPS2_VOLUME`. */
const CAPS2_VOLUME: number = 0x200000;

/** Bytes before a legacy header's texels: the magic and the header. */
const DATA_OFFSET: number = 128;

/** Bytes a block of four by four texels takes, by `fourCC`. */
const BLOCK_BYTES: Readonly<Record<string, number>> = { DXT1: 8, DXT3: 16, DXT5: 16 };

/**
 * A volume the texture reader refuses, since no surface draws one, read as `rain_patch_normal` samples it: its top
 * level, its blocks decoded on the CPU, because a GPU samples a compressed volume only with a feature it may lack.
 *
 * @param bytes - The file as stored.
 * @returns The volume, or null for a file that is no DXT1, DXT3 or DXT5 volume.
 */
export function readDdsVolume(bytes: ArrayBuffer): Nullable<IDdsVolume> {
  if (bytes.byteLength < DATA_OFFSET) {
    return null;
  }

  const view: DataView = new DataView(bytes);
  const fourCC: string = String.fromCharCode(...new Uint8Array(bytes, 84, 4));
  const blockBytes: number | undefined = BLOCK_BYTES[fourCC];

  if (view.getUint32(0, true) !== MAGIC || !(view.getUint32(112, true) & CAPS2_VOLUME) || !blockBytes) {
    return null;
  }

  const height: number = view.getUint32(12, true);
  const width: number = view.getUint32(16, true);
  const depth: number = Math.max(view.getUint32(24, true), 1);
  const across: number = Math.max(Math.ceil(width / 4), 1);
  const down: number = Math.max(Math.ceil(height / 4), 1);

  if (DATA_OFFSET + across * down * depth * blockBytes > bytes.byteLength) {
    return null;
  }

  const source: Uint8Array = new Uint8Array(bytes, DATA_OFFSET);
  const rgba: Uint8Array = new Uint8Array(width * height * depth * 4);
  const block: Uint8Array = new Uint8Array(64);

  for (let slice: number = 0; slice < depth; slice += 1) {
    for (let row: number = 0; row < down; row += 1) {
      for (let column: number = 0; column < across; column += 1) {
        const at: number = ((slice * down + row) * across + column) * blockBytes;

        decodeBlock(source, at, fourCC, block);

        for (let y: number = 0; y < 4 && row * 4 + y < height; y += 1) {
          for (let x: number = 0; x < 4 && column * 4 + x < width; x += 1) {
            const to: number = ((slice * height + row * 4 + y) * width + column * 4 + x) * 4;

            rgba.set(block.subarray((y * 4 + x) * 4, (y * 4 + x) * 4 + 4), to);
          }
        }
      }
    }
  }

  return { depth, height, rgba, width };
}

/** One block's sixteen texels, four bytes each, row by row. */
function decodeBlock(source: Uint8Array, at: number, fourCC: string, out: Uint8Array): void {
  const colorAt: number = fourCC === "DXT1" ? at : at + 8;

  decodeColors(source, colorAt, fourCC === "DXT1", out);

  if (fourCC === "DXT5") {
    decodeInterpolatedAlpha(source, at, out);
  } else if (fourCC === "DXT3") {
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
