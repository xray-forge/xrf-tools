import {
  CompressedPixelFormat,
  CompressedTexture,
  CompressedTextureMipmap,
  LinearFilter,
  RepeatWrapping,
  RGB_S3TC_DXT1_Format,
  RGBA_S3TC_DXT1_Format,
  RGBAFormat,
  Texture,
} from "three";
import { DDS, DDSLoader } from "three/examples/jsm/loaders/DDSLoader.js";

import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { XrayAsset, XrayResolution } from "@/core/ipc/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/ipc/types/xrf-visual";
import { Nullable, Optional } from "@/lib/types/general";

/** Shared parser, since `DDSLoader.parse` keeps no state between calls and constructing one per texture is waste. */
const DDS_LOADER: DDSLoader = new DDSLoader();

/**
 * Why a submesh ended up without a texture on screen, or that it has one.
 */
export enum EVisualTextureState {
  /** The submesh declares no texture, which is normal for a skeleton's own record. */
  ABSENT = "absent",
  /** Bytes are still on the way. */
  LOADING = "loading",
  /** Uploaded and applied, in the layout the file stores. */
  APPLIED = "applied",
  /**
   * Applied, but expanded by the backend first because the renderer cannot upload this layout.
   *
   * Distinct from `APPLIED` because the upload is not the file: it arrives as one png, so it carries no mip chain
   * whatever the file's header says, and costs the memory of raw pixels rather than of blocks.
   */
  DECODED = "decoded",
  /** Located, but stored in a format neither the renderer nor the backend can read. */
  UNSUPPORTED_FORMAT = "unsupportedFormat",
  /** Nothing to load: no source was searchable, or neither the reference nor the engine's dummy resolved. */
  UNRESOLVED = "unresolved",
  /** Located, but reading or parsing the file failed, or the reference was not a usable one. */
  FAILED = "failed",
}

/**
 *  What became of one submesh's texture on the frontend, paired with what the backend resolved.
 */
export interface IVisualTextureStatus {
  submeshIndex: number;
  state: EVisualTextureState;
  /** Present when the state is `FAILED`, so a panel can say why rather than only that. */
  reason: Nullable<string>;
}

/** A submesh texture whose bytes can be fetched, and the located file to fetch them from. */
export interface ILoadableTexture {
  submeshIndex: number;
  logicalPath: string;
}

/**
 * Submeshes worth fetching bytes for, paired with the logical path to fetch.
 *
 * The path comes from the outcome rather than from the reference, so the read lands on the file resolution named — a
 * substituted dummy included — instead of resolving a second time and possibly differently.
 */
export function toLoadableTextures(textures: Array<VisualTextureDependency>): Array<ILoadableTexture> {
  return textures.flatMap((texture) => {
    const asset: Nullable<XrayAsset> = getLocatedAsset(texture.resolution);

    return asset ? [{ submeshIndex: texture.submeshIndex, logicalPath: asset.logicalPath }] : [];
  });
}

/**
 * The state a submesh starts in, before any bytes are asked for.
 *
 * A rejected reference is a failure rather than an absence: the name in the mesh header is unusable, which is worth
 * saying rather than showing the submesh as having nothing to load.
 */
export function toInitialTextureState(resolution: XrayResolution): EVisualTextureState {
  if (getLocatedAsset(resolution)) {
    return EVisualTextureState.LOADING;
  }

  return resolution.kind === "rejected" ? EVisualTextureState.FAILED : EVisualTextureState.UNRESOLVED;
}

/**
 * Turn DDS bytes into an uploadable texture, or say that three.js cannot.
 *
 * `DDSLoader` refuses two ways and both look the same from outside: an unknown `DXGI_FORMAT` under a DX10 header
 * logs to the console and falls through, and an uncompressed layout whose channel masks are not BGRA matches neither
 * of its two uncompressed branches. Either way it returns a parse with a null format, so checking that covers both -
 * and covers the next format it has not learnt yet. Both occur in the reference trees: `BC7_UNorm` in Gunslinger and
 * `A8B8G8R8` in Anomaly.
 *
 * Assembly follows `CompressedTextureLoader`'s own single-file path rather than improvising, because one of its
 * steps is load-bearing: a texture carrying no mip chain must drop to `LinearFilter`, or webgl samples an incomplete
 * texture and renders black. Not an edge case here - 1,805 of Anomaly's 2,197 distinct textures ship without mips.
 *
 * @param bytes - The file as read.
 * @param isAlphaRead - Whether any surface drawn with this file samples its alpha channel; see {@link toDdsFormat}.
 * @returns The texture, or null when three.js cannot upload this file.
 */
export function createDdsTexture(bytes: ArrayBuffer, isAlphaRead: boolean = false): Nullable<CompressedTexture> {
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
    toDdsFormat(parsed.format as CompressedPixelFormat, isAlphaRead)
  );

  // X-Ray samples base diffuse with wrap addressing: `r_Sampler` defaults to `D3DTADDRESS_WRAP`
  // (`Layers/xrRender/Blender_Recorder.h`) and the model blender overrides nothing. three.js defaults to clamp, which
  // smears the edge texel across every face whose uv leaves [0,1].
  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;

  if (parsed.mipmapCount === 1) {
    texture.minFilter = LinearFilter;
  }

  texture.needsUpdate = true;

  return texture;
}

/**
 * The upload format for a parsed file, recovering DXT1's one bit of alpha for the surfaces that read it.
 *
 * `DDSLoader` maps every `DXT1` fourcc to `RGB_S3TC_DXT1_Format`, which tells webgl to ignore the alpha bit the block
 * format carries. That is right for the vast majority of files and wrong for X-Ray's `tfADXT1`.
 *
 * The block layout of the two formats is identical, so this is a reinterpretation and not a conversion. It is keyed on
 * the surface rather than applied always because the transparent-black block mode occurs in files authored as opaque
 * too, and reading those as `RGBA` would punch holes in surfaces the engine draws solid.
 *
 * @param format - What `DDSLoader` reported.
 * @param isAlphaRead - Whether any surface drawn with this file samples its alpha channel.
 * @returns The format to upload with.
 */
function toDdsFormat(format: CompressedPixelFormat, isAlphaRead: boolean): CompressedPixelFormat {
  return isAlphaRead && format === RGB_S3TC_DXT1_Format ? RGBA_S3TC_DXT1_Format : format;
}

/** A texture's top mip on the cpu, for a layout that stores its texels plainly. */
export interface IVisualTextureTexels {
  width: number;
  height: number;
  /** Rgba bytes, row major, the row X-Ray stores first coming first. */
  data: Uint8Array;
}

/**
 * Reads a dds file's top mip back as plain texels, when the file stores them plainly.
 *
 * Only an uncompressed layout answers. A block-compressed one would have to be decoded to be read texel by texel, and
 * a decoder here would be a second implementation of something the gpu already does correctly.
 *
 * Parsed a second time rather than handed out by {@link createDdsTexture}, because almost nothing wants this: one
 * surface reading two small files for a hover readout should not put a cpu copy in the path of every model upload.
 *
 * @param bytes - The file as read.
 * @returns Its top mip, or null for a layout stored as blocks.
 */
export function readDdsTexels(bytes: ArrayBuffer): Nullable<IVisualTextureTexels> {
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
export function readVisualTexel(texels: IVisualTextureTexels, x: number, y: number): [number, number, number, number] {
  const column: number = Math.min(Math.max(x, 0), texels.width - 1);
  const row: number = Math.min(Math.max(y, 0), texels.height - 1);
  const at: number = (row * texels.width + column) * 4;

  return [texels.data[at] / 255, texels.data[at + 1] / 255, texels.data[at + 2] / 255, texels.data[at + 3] / 255];
}

/**
 * Turn decoded png bytes into an uploadable texture, for a file three.js would not read itself.
 *
 * The fallback path, reached only when {@link createDdsTexture} refuses: the backend decodes what that loader declines
 * and hands back a png the webview reads natively. Between `image_dds` and its own mask expansion, that covers every
 * layout in the reference trees - `BC7_UNorm`, RGBA-ordered `A8B8G8R8`, `ATI2`, `A8` alpha-only, `R5G6B5`,
 * alpha-luminance and `X8R8G8B8` - so a submesh is left plain only when the file itself cannot be read.
 *
 * A decoded texture carries **no mip chain**, since a png is one image: it is sampled with a linear filter below, and
 * it will shimmer at distance where its compressed neighbours do not.
 *
 * `flipY` is set false to match the compressed path, which never flips: X-Ray stores rows top first, and a texture that
 * disagreed with the rest would be the only one on the model rendered upside down.
 *
 * @param bytes - Png bytes as the backend decoded them.
 * @returns An uploadable texture.
 */
export async function createDecodedTexture(bytes: ArrayBuffer): Promise<Texture> {
  const bitmap: ImageBitmap = await createImageBitmap(new Blob([bytes], { type: "image/png" }));
  const texture: Texture = new Texture(bitmap);

  texture.wrapS = RepeatWrapping;
  texture.wrapT = RepeatWrapping;
  texture.flipY = false;
  // A decoded png carries no mip chain, and an incomplete texture samples black without this.
  texture.minFilter = LinearFilter;
  texture.needsUpdate = true;

  return texture;
}
