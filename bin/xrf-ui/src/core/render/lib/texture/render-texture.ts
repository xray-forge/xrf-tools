import { Nullable } from "@xrf/types";
import {
  ClampToEdgeWrapping,
  CompressedTexture,
  DataTexture,
  LinearFilter,
  LinearMipmapLinearFilter,
  LinearMipmapNearestFilter,
  NearestFilter,
  NearestMipmapLinearFilter,
  NearestMipmapNearestFilter,
  RepeatWrapping,
  RGB_S3TC_DXT1_Format,
  RGBAFormat,
  SRGBColorSpace,
  Texture,
} from "three";

import { EDdsLayout, IDdsFile, IDdsRead, IDdsRefusal, readDdsFile } from "@/core/render/lib/dds";

/**
 * How many samples the engine takes across a texture seen at an angle, `ps_r__tf_Anisotropic`
 * (`Layers/xrRender/xrRender_console.cpp`), which the console offers between one and sixteen.
 */
export const XRAY_TEXTURE_ANISOTROPY: number = 8;

/**
 * What a file has to survive upload with, which the file cannot answer for itself.
 */
export interface IRenderTextureOptions {
  /** Whether any surface drawn with this file samples its alpha channel. */
  isAlphaRead?: boolean;
  /** Whether the file holds colour rather than numbers. */
  isColor?: boolean;
  /** Whether the surfaces drawn with it sample its mip chain, which a wall mark does not. */
  isMipped?: boolean;
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
  const isMipped: boolean = options.isMipped ?? true;
  const texture: CompressedTexture = new CompressedTexture(
    // Only the level the engine samples, so an unmipped texture costs the gpu no chain it never reads.
    isMipped ? file.mipmaps : file.mipmaps.slice(0, 1),
    file.width,
    file.height,
    // The typings admit only a compressed format, even though three's own `CompressedTextureLoader` assigns
    // `RGBAFormat` to a `CompressedTexture` for exactly the uncompressed layouts the reader expands.
    file.layout.kind === EDdsLayout.BLOCK ? file.layout.format : (RGBAFormat as never)
  );

  // X-Ray samples base diffuse with wrap addressing: `r_Sampler` defaults to `D3DTADDRESS_WRAP`
  // (`Layers/xrRender/Blender_Recorder.h`) and the model blender overrides nothing. three.js defaults to clamp, which
  // smears the edge texel across every face whose uv leaves [0,1].
  // The wall mark sampler clamps where the base sampler wraps, and takes one sample where it takes eight.
  texture.wrapS = isMipped ? RepeatWrapping : ClampToEdgeWrapping;
  texture.wrapT = isMipped ? RepeatWrapping : ClampToEdgeWrapping;
  texture.anisotropy = isMipped ? XRAY_TEXTURE_ANISOTROPY : 1;

  if (options.isColor) {
    texture.colorSpace = SRGBColorSpace;
  }

  if (!isMipped || file.mipmapCount === 1) {
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
  texture.anisotropy = XRAY_TEXTURE_ANISOTROPY;
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

/**
 * Side of the generated checker, in texels. Small because it is tiled: what matters is that it reads as a pattern at
 * any distance, not that it carries detail.
 */
const CHECKER_SIZE: number = 16;

/** Magenta and black, the colours a missing texture has meant since before any of this. */
const CHECKER_COLORS: ReadonlyArray<ReadonlyArray<number>> = [
  [255, 0, 255],
  [16, 16, 16],
];

/**
 * Built once and shared by every texture that wraps it: the pixels never change, and generating them per reference
 * would be the same kilobyte again for each.
 */
let checkerData: Nullable<Uint8Array> = null;

function toCheckerData(): Uint8Array {
  if (checkerData) {
    return checkerData;
  }

  const data: Uint8Array = new Uint8Array(CHECKER_SIZE * CHECKER_SIZE * 4);

  for (let y = 0; y < CHECKER_SIZE; y += 1) {
    for (let x = 0; x < CHECKER_SIZE; x += 1) {
      const at: number = (y * CHECKER_SIZE + x) * 4;
      const [red, green, blue] = CHECKER_COLORS[((x >> 1) + (y >> 1)) % 2];

      data[at] = red;
      data[at + 1] = green;
      data[at + 2] = blue;
      // Fully opaque, because the surfaces that most need this are the cut-out ones: anything below their reference
      // would be discarded and the surface would go on saying nothing.
      data[at + 3] = 255;
    }
  }

  checkerData = data;

  return data;
}

/**
 * A checker to stand in for a texture that is missing or is not a texture.
 *
 * @returns An uploadable texture, owned by the caller and disposed with the rest.
 */
export function createCheckerTexture(): DataTexture {
  const texture: DataTexture = new DataTexture(toCheckerData(), CHECKER_SIZE, CHECKER_SIZE, RGBAFormat);

  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;
  // Nearest, so the squares stay squares: a filtered checker at distance averages to flat grey, which is the one
  // thing this must never look like.
  texture.magFilter = NearestFilter;
  texture.minFilter = NearestFilter;
  texture.colorSpace = SRGBColorSpace;
  texture.needsUpdate = true;

  return texture;
}

/**
 * Whether an uploaded texture carries an alpha channel at all.
 *
 * @param texture - An uploaded texture, or nothing uploaded.
 * @returns Whether anything drawing it can read alpha from it.
 */
export function hasRenderTextureAlpha(texture: Nullable<Texture>): boolean {
  return Boolean(texture) && texture?.format !== RGB_S3TC_DXT1_Format;
}

/** Three's filters by number, so a reader sees what the sampler does rather than a constant. */
const MIN_FILTERS: Readonly<Record<number, string>> = {
  [LinearFilter]: "linear",
  [LinearMipmapLinearFilter]: "linear between mips",
  [LinearMipmapNearestFilter]: "linear, nearest between mips",
  [NearestFilter]: "nearest",
  [NearestMipmapLinearFilter]: "nearest, linear between mips",
  [NearestMipmapNearestFilter]: "nearest between mips",
};

/**
 * How one texture was uploaded, in a line.
 *
 * @param texture - The texture the renderer holds.
 * @returns Its levels, filter, addressing and anisotropy.
 */
export function describeTextureUpload(texture: Texture): string {
  const levels: number = texture.mipmaps?.length || 1;
  const filter: string = MIN_FILTERS[texture.minFilter] ?? String(texture.minFilter);
  const wrap: string = texture.wrapS === ClampToEdgeWrapping ? "clamped" : "wrapped";

  return `${levels} ${levels === 1 ? "level" : "levels"} · ${filter} · ${wrap} · aniso ${texture.anisotropy}`;
}
