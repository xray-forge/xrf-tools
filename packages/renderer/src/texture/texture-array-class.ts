import { Nullable } from "@xrf/types";
import { CompressedArrayTexture, CompressedTexture, CubeTexture, Texture } from "three/webgpu";

/**
 * @param texture - A texture a key holds on the GPU.
 * @returns What an array holding it has to share with it: its block format, its size, its levels and how it is sampled;
 *   null for one no array holds, which is anything but a flat block-compressed texture of one layer.
 */
export function toTextureArrayClass(texture: Texture): Nullable<string> {
  const compressed: CompressedTexture = texture as CompressedTexture;

  if (
    !compressed.isCompressedTexture ||
    (texture as Partial<CubeTexture>).isCubeTexture ||
    (texture as Partial<CompressedArrayTexture>).isCompressedArrayTexture
  ) {
    return null;
  }

  const { width, height } = compressed.image as { width: number; height: number };

  return [
    compressed.format,
    width,
    height,
    compressed.mipmaps.length,
    compressed.colorSpace,
    compressed.minFilter,
    compressed.magFilter,
    compressed.anisotropy,
    compressed.wrapS,
    compressed.wrapT,
  ].join(":");
}
