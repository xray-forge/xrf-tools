import { clamp } from "@xrf/math";

/** Bytes ahead of the texels: the width and the height, each a little-endian `u32`. */
const HEADER_BYTES: number = 8;

/**
 * A texture's top level on the cpu, as plain texels.
 */
export interface ITextureTexels {
  width: number;
  height: number;
  /** Rgba bytes, row major, the row X-Ray stores first coming first. */
  data: Uint8Array;
}

/**
 * @param bytes - What `textures/read_texels` answered: the size, then four bytes a texel.
 * @returns The texels, viewed over those bytes rather than copied.
 */
export function readTextureTexels(bytes: ArrayBuffer): ITextureTexels {
  const header: DataView = new DataView(bytes, 0, HEADER_BYTES);
  const width: number = header.getUint32(0, true);
  const height: number = header.getUint32(4, true);

  if (bytes.byteLength !== HEADER_BYTES + width * height * 4) {
    throw new Error(`Texels of ${width} x ${height} cannot be ${bytes.byteLength - HEADER_BYTES} bytes long`);
  }

  return { data: new Uint8Array(bytes, HEADER_BYTES), height, width };
}

/**
 * One texel of a cpu copy, in the range a shader reads.
 *
 * @param texels - The level to read.
 * @param x - Column, from the left.
 * @param y - Row, from the top, as the file stores them.
 * @returns Its four channels, each in `[0, 1]`.
 */
export function readTextureTexel(texels: ITextureTexels, x: number, y: number): [number, number, number, number] {
  const column: number = clamp(x, 0, texels.width - 1);
  const row: number = clamp(y, 0, texels.height - 1);
  const at: number = (row * texels.width + column) * 4;

  return [texels.data[at] / 255, texels.data[at + 1] / 255, texels.data[at + 2] / 255, texels.data[at + 3] / 255];
}
