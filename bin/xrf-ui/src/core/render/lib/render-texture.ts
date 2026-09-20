import {
  CompressedPixelFormat,
  CompressedTexture,
  CompressedTextureMipmap,
  LinearFilter,
  RepeatWrapping,
  RGB_S3TC_DXT1_Format,
  RGBA_S3TC_DXT1_Format,
  RGBAFormat,
  SRGBColorSpace,
  Texture,
} from "three";
import { DDS, DDSLoader } from "three/examples/jsm/loaders/DDSLoader.js";

import { Nullable, Optional } from "@/lib/types/general";

/** Shared parser, since `DDSLoader.parse` keeps no state between calls and constructing one per texture is waste. */
const DDS_LOADER: DDSLoader = new DDSLoader();

/**
 * What a file has to survive upload with, which the file cannot answer for itself.
 */
export interface IRenderTextureOptions {
  /**
   * Whether any surface drawn with this file samples its alpha channel; see {@link toDdsFormat}.
   */
  isAlphaRead?: boolean;
  /**  Whether the file holds colour rather than numbers. */
  isColor?: boolean;
}

/**
 * Turn DDS bytes into an uploadable texture, or say that three.js cannot.
 *
 * @param bytes - The file as read.
 * @param options - What the file has to survive upload with.
 * @returns The texture, or null when three.js cannot upload this file.
 */
export function createDdsTexture(bytes: ArrayBuffer, options: IRenderTextureOptions = {}): Nullable<CompressedTexture> {
  const parsed: DDS = DDS_LOADER.parse(bytes, true);

  // The declared type is not nullable, but the parser initialises `format` to null and leaves it there when it refuses.
  if (parsed.format === null || parsed.mipmaps.length === 0) {
    return null;
  }

  // A cubemap needs its faces split apart, which no model texture requires; rendering it flat would show one face
  // stretched over the mesh, so it is refused rather than guessed at.
  if (parsed.isCubemap) {
    return null;
  }

  const texture: CompressedTexture = new CompressedTexture(
    parsed.mipmaps,
    parsed.width,
    parsed.height,
    // `DDSLoader` reports `RGBAFormat` for an uncompressed file, which the typings do not admit here even though
    // three's own `CompressedTextureLoader` assigns exactly that to a `CompressedTexture`.
    toDdsFormat(parsed.format as CompressedPixelFormat, options.isAlphaRead ?? false)
  );

  // X-Ray samples base diffuse with wrap addressing: `r_Sampler` defaults to `D3DTADDRESS_WRAP`
  // (`Layers/xrRender/Blender_Recorder.h`) and the model blender overrides nothing. three.js defaults to clamp, which
  // smears the edge texel across every face whose uv leaves [0,1].
  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;

  if (options.isColor) {
    texture.colorSpace = SRGBColorSpace;
  }

  if (parsed.mipmapCount === 1) {
    texture.minFilter = LinearFilter;
  }

  texture.needsUpdate = true;

  return texture;
}

/**
 * The upload format for a parsed file, recovering DXT1's one bit of alpha for the surfaces that read it.
 *
 * @param format - What `DDSLoader` reported.
 * @param isAlphaRead - Whether any surface drawn with this file samples its alpha channel.
 * @returns The format to upload with.
 */
function toDdsFormat(format: CompressedPixelFormat, isAlphaRead: boolean): CompressedPixelFormat {
  return isAlphaRead && format === RGB_S3TC_DXT1_Format ? RGBA_S3TC_DXT1_Format : format;
}

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
  const parsed: DDS = DDS_LOADER.parse(bytes, true);
  const mip: Optional<CompressedTextureMipmap> = parsed.mipmaps[0];

  // `DDSLoader` reports `RGBAFormat` only for a file it expanded rather than left as blocks.
  if (!mip || parsed.isCubemap || (parsed.format as number) !== (RGBAFormat as number)) {
    return null;
  }

  return { data: new Uint8Array(mip.data), height: mip.height, width: mip.width };
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
 * Turn decoded png bytes into an uploadable texture, for a file three.js would not read itself.
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
