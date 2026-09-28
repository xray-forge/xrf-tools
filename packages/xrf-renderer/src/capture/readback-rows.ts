/** The row alignment WebGPU copies a texture into a buffer with. */
export const READBACK_ROW_ALIGNMENT: number = 256;

/**
 * Drops the padding WebGPU copies each row with, where the width leaves some.
 *
 * @param pixels - Rows as read back.
 * @param width - Texels in a row.
 * @param height - Rows.
 * @returns Tightly packed rgba rows.
 */
export function unpadReadbackRows(pixels: Uint8Array, width: number, height: number): Uint8ClampedArray<ArrayBuffer> {
  const row: number = width * 4;
  const stride: number = Math.ceil(row / READBACK_ROW_ALIGNMENT) * READBACK_ROW_ALIGNMENT;
  const packed: Uint8ClampedArray<ArrayBuffer> = new Uint8ClampedArray(row * height);

  if (pixels.length === packed.length) {
    packed.set(pixels);

    return packed;
  }

  for (let y = 0; y < height; y += 1) {
    packed.set(pixels.subarray(y * stride, y * stride + row), y * row);
  }

  return packed;
}
