import {
  CompressedPixelFormat,
  CompressedTexture,
  LinearFilter,
  RepeatWrapping,
  RGBAFormat,
  SRGBColorSpace,
  Texture,
} from "three";

import { EDdsLayout, IDdsFile, IDdsMipmap, readDdsFile, TDdsRead } from "@/core/render/lib/dds";
import { Nullable, Optional } from "@/lib/types/general";

/**
 * What a file has to survive upload with, which the file cannot answer for itself.
 */
export interface IRenderTextureOptions {
  /** Whether any surface drawn with this file samples its alpha channel. */
  isAlphaRead?: boolean;
  /** Whether the file holds colour rather than numbers. */
  isColor?: boolean;
}

/**
 * What became of one upload: the texture, or why the file could not be uploaded as it is stored.
 */
export interface IRenderTextureUpload {
  texture: Nullable<CompressedTexture>;
  /** Present exactly when `texture` is null, naming the layout that was refused. */
  refusal: Nullable<string>;
}

/**
 * Turn DDS bytes into an uploadable texture, or say why not.
 *
 * @param bytes - The file as read.
 * @param options - What the file has to survive upload with.
 * @returns The texture, or the reason there is none.
 */
export function createDdsTexture(bytes: ArrayBuffer, options: IRenderTextureOptions = {}): IRenderTextureUpload {
  const read: TDdsRead = readDdsFile(bytes, options.isAlphaRead ?? false);

  if (read.kind === "refused") {
    return { refusal: `${read.reason}: ${read.detail}`, texture: null };
  }

  const file: IDdsFile = read.file;

  // A cubemap needs its faces split apart, which no surface of a level or a model requires; drawing it flat would
  // show one face stretched over the mesh, so it is refused rather than guessed at.
  if (file.isCubemap) {
    return { refusal: "incompleteCubemap: a cubemap is not drawn on a surface", texture: null };
  }

  if (!file.mipmaps.length) {
    return { refusal: "truncated: the file carries no mip to upload", texture: null };
  }

  const texture: CompressedTexture = new CompressedTexture(
    // `CompressedTexture` takes an expanded rgba mip the same way three's own loader hands it one.
    file.mipmaps as Array<CompressedTextureMipmapLike>,
    file.width,
    file.height,
    // The typings admit only a compressed format here, even though three's own `CompressedTextureLoader` assigns
    // `RGBAFormat` to a `CompressedTexture` for exactly the uncompressed layouts this expands.
    (file.layout.kind === EDdsLayout.BLOCK ? file.layout.format : RGBAFormat) as CompressedPixelFormat
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
 * The shape three's `CompressedTexture` takes its mips in, which is what {@link IDdsMipmap} already is.
 */
type CompressedTextureMipmapLike = IDdsMipmap;

/** A texture's top mip on the cpu, for a layout that stores its texels plainly. */
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
  const read: TDdsRead = readDdsFile(bytes);

  if (read.kind === "refused" || read.file.layout.kind !== EDdsLayout.TEXELS) {
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
