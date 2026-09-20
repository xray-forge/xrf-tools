import { CompressedTexture, LinearFilter, RepeatWrapping, RGBAFormat, SRGBColorSpace, Texture } from "three";

import { EDdsLayout, IDdsFile, IDdsRead, IDdsRefusal, readDdsFile } from "@/core/render/lib/dds";
import { Nullable } from "@/lib/types/general";

/**
 * What a file has to survive upload with, which the file cannot answer for itself.
 */
export interface IRenderTextureOptions {
  /** Whether any surface drawn with this file samples its alpha channel. */
  isAlphaRead?: boolean;
  /** Whether the file holds colour rather than numbers. */
  isColor?: boolean;
}

/** What became of one upload: exactly one of the two is present. */
export interface IRenderTextureUpload {
  texture: Nullable<CompressedTexture>;
  refusal: Nullable<IDdsRefusal>;
}

/**
 * Turn DDS bytes into an uploadable texture, or say why not.
 *
 * @param bytes - The file as read.
 * @param options - What the file has to survive upload with.
 * @returns The texture, or the reason there is none.
 */
export function createDdsTexture(bytes: ArrayBuffer, options: IRenderTextureOptions = {}): IRenderTextureUpload {
  const read: IDdsRead = readDdsFile(bytes, options.isAlphaRead ?? false);

  if (!read.file) {
    return { refusal: read.refusal, texture: null };
  }

  const file: IDdsFile = read.file;
  const texture: CompressedTexture = new CompressedTexture(
    file.mipmaps,
    file.width,
    file.height,
    // The typings admit only a compressed format, even though three's own `CompressedTextureLoader` assigns
    // `RGBAFormat` to a `CompressedTexture` for exactly the uncompressed layouts the reader expands.
    file.layout.kind === EDdsLayout.BLOCK ? file.layout.format : (RGBAFormat as never)
  );

  // X-Ray samples base diffuse with wrap addressing: `r_Sampler` defaults to `D3DTADDRESS_WRAP`
  // (`Layers/xrRender/Blender_Recorder.h`) and the model blender overrides nothing. three.js defaults to clamp, which
  // smears the edge texel across every face whose uv leaves [0,1].
  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;

  if (options.isColor) {
    texture.colorSpace = SRGBColorSpace;
  }

  if (file.mipmapCount === 1) {
    texture.minFilter = LinearFilter;
  }

  texture.needsUpdate = true;

  return { refusal: null, texture };
}

/**
 * Turn decoded png bytes into an uploadable texture, for a file the reader will not take.
 *
 * @param bytes - Png bytes as the backend decoded them.
 * @param options - What the file has to survive upload with; only its colour answer applies, since a png carries its
 *   own alpha and needs no format reinterpretation.
 * @returns An uploadable texture that closes its owned bitmap when disposed.
 */
export async function createDecodedTexture(bytes: ArrayBuffer, options: IRenderTextureOptions = {}): Promise<Texture> {
  const bitmap: ImageBitmap = await createImageBitmap(new Blob([bytes], { type: "image/png" }));
  const texture: Texture = new Texture(bitmap);

  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;
  texture.flipY = false;

  if (options.isColor) {
    texture.colorSpace = SRGBColorSpace;
  }

  // A decoded png carries no mip chain, and an incomplete texture samples black without this.
  texture.minFilter = LinearFilter;
  texture.needsUpdate = true;

  texture.addEventListener("dispose", (): void => bitmap.close());

  return texture;
}
