import { Nullable } from "@xrf/types";
import {
  CompressedPixelFormat,
  CompressedTexture,
  DataTexture,
  LinearFilter,
  LinearMipmapLinearFilter,
  NoColorSpace,
  RED_GREEN_RGTC2_Format,
  RED_RGTC1_Format,
  RepeatWrapping,
  RGB_BPTC_SIGNED_Format,
  RGB_BPTC_UNSIGNED_Format,
  RGBA_BPTC_Format,
  RGBA_S3TC_DXT1_Format,
  RGBA_S3TC_DXT3_Format,
  RGBA_S3TC_DXT5_Format,
  RGBAFormat,
  SIGNED_RED_GREEN_RGTC2_Format,
  SIGNED_RED_RGTC1_Format,
  Texture,
  UnsignedByteType,
} from "three/webgpu";

import { EDdsBlockFormat } from "#/texture/dds/dds-block-format";
import { IDdsFile, IDdsRead, readDdsFile } from "#/texture/dds/dds-file";
import { EDdsLayout } from "#/texture/dds/dds-layout";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";

/** The engine's `ps_r__tf_Anisotropic` default (`Layers/xrRender/xrRender_console.cpp`). */
export const XRAY_TEXTURE_ANISOTROPY: number = 8;

/** What a dds upload came to: exactly one of the two is present. */
export interface IRendererTextureUpload {
  texture: Nullable<Texture>;
  refusal: Nullable<IDdsRefusal>;
}

/**
 * Uploads a dds file as the engine samples it: raw bytes, never decoded from srgb, with the file's own mip chain.
 *
 * @param bytes - The file as read.
 * @returns The texture, or the reason the file cannot be read.
 */
export function createRendererTexture(bytes: ArrayBuffer): IRendererTextureUpload {
  const read: IDdsRead = readDdsFile(bytes);

  if (!read.file) {
    return { refusal: read.refusal, texture: null };
  }

  const file: IDdsFile = read.file;
  const texture: Texture =
    file.layout.kind === EDdsLayout.BLOCK
      ? new CompressedTexture(file.mipmaps, file.width, file.height, toCompressedFormat(file.layout.format))
      : createTexelTexture(file);

  describeSampling(texture, file.mipmaps.length);

  return { refusal: null, texture };
}

/**
 * Uploads a picture the backend decoded, for a file the reader will not take, with its bytes left exactly as stored.
 *
 * @param bytes - The encoded picture.
 * @param type - Its media type, such as `image/png`.
 * @returns The texture.
 */
export async function createRendererImageTexture(bytes: ArrayBuffer, type: string): Promise<Texture> {
  // Nothing converted: a browser otherwise applies the picture's colour profile and premultiplies alpha, and either
  // changes the bytes the engine would have sampled.
  const bitmap: ImageBitmap = await createImageBitmap(new Blob([bytes], { type }), {
    colorSpaceConversion: "none",
    imageOrientation: "none",
    premultiplyAlpha: "none",
  });
  const texture: Texture = new Texture(bitmap);

  texture.flipY = false;
  describeSampling(texture, 1);

  return texture;
}

/** A texel layout, already expanded to rgba by the reader, with its chain as three's mipmap list. */
function createTexelTexture(file: IDdsFile): Texture {
  const top = file.mipmaps[0];
  const texture: DataTexture = new DataTexture(top.data, top.width, top.height, RGBAFormat, UnsignedByteType);

  texture.mipmaps = file.mipmaps.length > 1 ? (file.mipmaps as unknown as Array<ImageData>) : [];

  return texture;
}

/** The engine's base sampler: wrap addressing, anisotropic, trilinear where the file carries a chain. */
function describeSampling(texture: Texture, levels: number): void {
  texture.colorSpace = NoColorSpace;
  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;
  texture.anisotropy = XRAY_TEXTURE_ANISOTROPY;
  texture.generateMipmaps = false;
  texture.magFilter = LinearFilter;
  texture.minFilter = levels > 1 ? LinearMipmapLinearFilter : LinearFilter;
  texture.needsUpdate = true;
}

/** The three constant WebGPU uploads a block layout under; its transfer is linear, so each lands on `-unorm`. */
function toCompressedFormat(format: EDdsBlockFormat): CompressedPixelFormat {
  switch (format) {
    case EDdsBlockFormat.BC1:
      return RGBA_S3TC_DXT1_Format;

    case EDdsBlockFormat.BC2:
      return RGBA_S3TC_DXT3_Format;

    case EDdsBlockFormat.BC3:
      return RGBA_S3TC_DXT5_Format;

    case EDdsBlockFormat.BC4:
      return RED_RGTC1_Format;

    case EDdsBlockFormat.BC4_SIGNED:
      return SIGNED_RED_RGTC1_Format;

    case EDdsBlockFormat.BC5:
      return RED_GREEN_RGTC2_Format;

    case EDdsBlockFormat.BC5_SIGNED:
      return SIGNED_RED_GREEN_RGTC2_Format;

    case EDdsBlockFormat.BC6H:
      return RGB_BPTC_UNSIGNED_Format;

    case EDdsBlockFormat.BC6H_SIGNED:
      return RGB_BPTC_SIGNED_Format;

    case EDdsBlockFormat.BC7:
      return RGBA_BPTC_Format;
  }
}
