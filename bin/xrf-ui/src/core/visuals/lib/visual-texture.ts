import { CompressedPixelFormat, CompressedTexture, LinearFilter, RepeatWrapping, Texture } from "three";
import { DDS, DDSLoader } from "three/examples/jsm/loaders/DDSLoader.js";

import { XrayAsset, XrayResolution } from "@/core/bindings/types/xrf-vfs";
import { VisualTextureDependency } from "@/core/bindings/types/xrf-visual";
import { Nullable } from "@/lib/types/general";

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
  /** Uploaded and applied. */
  APPLIED = "applied",
  /** Located, but stored in a format three.js cannot upload. */
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

/**
 * The asset a resolution located, or null when it located none.
 *
 * A substituted reference counts: the engine's dummy is a real file and rendering it is what the game does, which
 * is the point of substituting rather than leaving the submesh blank.
 */
export function getLocatedAsset(resolution: XrayResolution): Nullable<XrayAsset> {
  return listLocatedAssets(resolution)[0] ?? null;
}

/**
 * Every asset an outcome located, which for a masked reference is more than one.
 *
 * A texture reference answers with exactly one file, but a motion reference may be a mask - `wpn\wpn_ak74_*.omf` - so
 * naming only the first would misreport what was found.
 *
 * @param resolution - What the backend reported for one reference.
 * @returns The located assets, empty when the outcome located nothing.
 */
export function listLocatedAssets(resolution: XrayResolution): Array<XrayAsset> {
  return resolution.kind === "resolved" || resolution.kind === "substituted" ? resolution.assets : [];
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
 * @returns The texture, or null when three.js cannot upload this file.
 */
export function createDdsTexture(bytes: ArrayBuffer): Nullable<CompressedTexture> {
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
    parsed.format as CompressedPixelFormat
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
