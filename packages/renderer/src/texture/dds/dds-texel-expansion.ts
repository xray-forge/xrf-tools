import { EDdsChannels, getDdsSourceStride } from "#/texture/dds/dds-channels";

/** Texels expand to four bytes however few the file stored. */
const EXPANDED_STRIDE: number = 4;

/**
 * Expands one stored mip into rgba byte order.
 *
 * @param bytes - The file.
 * @param offset - Where this mip starts in it.
 * @param width - Its width in texels.
 * @param height - Its height in texels.
 * @param channels - The order the file keeps its channels in.
 * @returns Four bytes a texel, red first.
 */
export function expandDdsTexels(
  bytes: ArrayBuffer,
  offset: number,
  width: number,
  height: number,
  channels: EDdsChannels
): Uint8Array {
  const stride: number = getDdsSourceStride(channels);
  const source: Uint8Array = new Uint8Array(bytes, offset, width * height * stride);

  if (channels === EDdsChannels.RGBA) {
    // Already rgba, so the expansion is a copy rather than a reorder.
    return new Uint8Array(source);
  }

  const expanded: Uint8Array = new Uint8Array(width * height * EXPANDED_STRIDE);
  const hasAlpha: boolean = channels === EDdsChannels.BGRA;

  for (let texel = 0; texel < width * height; texel += 1) {
    const from: number = texel * stride;
    const to: number = texel * EXPANDED_STRIDE;

    expanded[to] = source[from + 2];
    expanded[to + 1] = source[from + 1];
    expanded[to + 2] = source[from];
    // A layout storing three bytes says nothing about alpha, and a texture that read zero there would be invisible.
    expanded[to + 3] = hasAlpha ? source[from + 3] : 255;
  }

  return expanded;
}
