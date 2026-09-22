import { Nullable } from "@xrf/types";

import { EDdsLayout, getDdsStoredLength, TDdsLayout } from "#/texture/dds/dds-layout";
import { expandDdsTexels } from "#/texture/dds/dds-texel-expansion";

/** One mip: block bytes untouched, or texels already expanded to rgba. */
export interface IDdsMipmap {
  data: Uint8Array;
  width: number;
  height: number;
}

/** What a mip walk needs to know about the file it is walking. */
export interface IDdsMipmapChain {
  width: number;
  height: number;
  mipmapCount: number;
  layout: TDdsLayout;
}

/**
 * Walks a mip chain, halving until the chain is spent.
 *
 * A block layout is handed out as a view over the file rather than a copy: the blocks go to the gpu exactly as stored,
 * and copying them would double what a texture costs on the way in.
 *
 * @param bytes - The file.
 * @param start - Where the first mip starts.
 * @param chain - The size, count and layout the header declares.
 * @returns The mips, largest first, or null when the file stops before the texels its header declares.
 */
export function readDdsMipmaps(bytes: ArrayBuffer, start: number, chain: IDdsMipmapChain): Nullable<Array<IDdsMipmap>> {
  const mipmaps: Array<IDdsMipmap> = [];

  let offset: number = start;
  let width: number = chain.width;
  let height: number = chain.height;

  for (let level = 0; level < chain.mipmapCount; level += 1) {
    const stored: number = getDdsStoredLength(chain.layout, width, height);

    if (offset + stored > bytes.byteLength) {
      return null;
    }

    mipmaps.push({
      data:
        chain.layout.kind === EDdsLayout.BLOCK
          ? new Uint8Array(bytes, offset, stored)
          : expandDdsTexels(bytes, offset, width, height, chain.layout.channels),
      height,
      width,
    });

    offset += stored;
    width = Math.max(width >> 1, 1);
    height = Math.max(height >> 1, 1);
  }

  return mipmaps;
}
