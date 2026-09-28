import { ERendererTextureEncoding, TRendererTextureSource } from "@xrf/renderer";

/**
 * @param bytes - A texture's file as it was read, or the backend's picture of it; moved to the renderer.
 * @param isDecoded - Whether they are the backend's png rather than the file.
 * @returns What the renderer uploads.
 */
export function toRendererTextureSource(bytes: ArrayBuffer, isDecoded: boolean): TRendererTextureSource {
  return isDecoded
    ? { bytes, encoding: ERendererTextureEncoding.IMAGE, type: "image/png" }
    : { bytes, encoding: ERendererTextureEncoding.DDS };
}

/**
 * A checker, opaque and sampled nearest, so it reads as a pattern at any distance.
 *
 * @param size - Its side, in texels.
 * @param colors - Its two colours, as bytes.
 * @param cell - Texels each square is across.
 * @returns Its texels.
 */
export function createRenderCheckerSource(
  size: number,
  colors: readonly [readonly [number, number, number], readonly [number, number, number]],
  cell: number = 1
): TRendererTextureSource {
  const bytes: Uint8Array<ArrayBuffer> = new Uint8Array(size * size * 4);

  for (let y: number = 0; y < size; y += 1) {
    for (let x: number = 0; x < size; x += 1) {
      bytes.set([...colors[(Math.floor(x / cell) + Math.floor(y / cell)) % 2], 0xff], (y * size + x) * 4);
    }
  }

  return { bytes: bytes.buffer, encoding: ERendererTextureEncoding.RGBA, height: size, isNearest: true, width: size };
}
