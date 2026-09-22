import { Nullable, Optional } from "@xrf/types";

import { EDdsLayout, IDdsMipmap, IDdsRead, readDdsFile } from "@/core/render/lib/dds";

/**
 * A texture's top mip on the cpu, for a layout that stores its texels plainly.
 */
export interface IRenderTextureTexels {
  width: number;
  height: number;
  /** Rgba bytes, row major, the row X-Ray stores first coming first. */
  data: Uint8Array;
}

/**
 * Reads a dds file's top mip back as plain texels, when the file stores them plainly.
 *
 * @param bytes - The file as read.
 * @returns Its top mip, or null for a layout stored as blocks.
 */
export function readDdsTexels(bytes: ArrayBuffer): Nullable<IRenderTextureTexels> {
  const read: IDdsRead = readDdsFile(bytes);

  if (!read.file || read.file.layout.kind !== EDdsLayout.TEXELS) {
    return null;
  }

  const mip: Optional<IDdsMipmap> = read.file.mipmaps[0];

  return mip ? { data: mip.data, height: mip.height, width: mip.width } : null;
}

/**
 * One texel of a cpu copy, in the range a shader reads.
 *
 * @param texels - The mip to read.
 * @param x - Column, from the left.
 * @param y - Row, from the top, as the file stores them.
 * @returns Its four channels, each in `[0, 1]`.
 */
export function readRenderTexel(texels: IRenderTextureTexels, x: number, y: number): [number, number, number, number] {
  const column: number = Math.min(Math.max(x, 0), texels.width - 1);
  const row: number = Math.min(Math.max(y, 0), texels.height - 1);
  const at: number = (row * texels.width + column) * 4;

  return [texels.data[at] / 255, texels.data[at + 1] / 255, texels.data[at + 2] / 255, texels.data[at + 3] / 255];
}
