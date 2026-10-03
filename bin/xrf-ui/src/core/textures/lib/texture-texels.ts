import { IDdsTexels } from "@xrf/dds";

/** Bytes ahead of the texels: the width and the height, each a little-endian `u32`. */
const HEADER_BYTES: number = 8;

/**
 * @param bytes - What `textures/read_texels` answered: the size, then four bytes a texel.
 * @returns The texels, viewed over those bytes rather than copied.
 */
export function readTextureTexels(bytes: ArrayBuffer): IDdsTexels {
  const header: DataView = new DataView(bytes, 0, HEADER_BYTES);
  const width: number = header.getUint32(0, true);
  const height: number = header.getUint32(4, true);

  if (bytes.byteLength !== HEADER_BYTES + width * height * 4) {
    throw new Error(`Texels of ${width} x ${height} cannot be ${bytes.byteLength - HEADER_BYTES} bytes long`);
  }

  return { data: new Uint8Array(bytes, HEADER_BYTES), height, width };
}
