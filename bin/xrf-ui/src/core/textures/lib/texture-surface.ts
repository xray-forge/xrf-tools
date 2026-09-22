import { Nullable } from "@xrf/types";

import { getLocatedAsset } from "@/core/assets/lib/resolution";
import { AssetTextureShape, TextureDescription } from "@/core/ipc/types/xrf-app";
import { XrayAsset } from "@/core/ipc/types/xrf-vfs";
import { IRenderTextureTexels } from "@/core/render/lib/texture/render-texels";

/**
 * One file the surface is drawn from, as it was read.
 */
export interface ITextureSurfaceFile {
  /** The file itself: a dds as it sits on disk, or the png the backend decoded a refused layout into. */
  bytes: ArrayBuffer;
  /** Whether those bytes are that decode rather than the file. */
  isDecoded: boolean;
  /** What the file measures, which only the side that read it can say for a layout nothing uploaded. */
  width: number;
  height: number;
}

/**
 * What one texture is drawn from, as data.
 */
export interface ITextureSurfaceFiles {
  /** The base file, or null for a descriptor with no texture beside it. */
  base: Nullable<ITextureSurfaceFile>;
  /** The pair the engine binds, or null for a material that binds none. */
  bump: Nullable<ITextureSurfaceBump>;
  /**
   * Width over height of the base file, so a flat body is drawn in the proportions the texture was authored in.
   */
  aspect: number;
}

/** The two files of a bump pair, which are only ever read and drawn together. */
export interface ITextureSurfaceBump {
  bump: ITextureSurfaceFile;
  companion: ITextureSurfaceFile;
}

/** The bodies a texture can be laid on, each answering a different question about it. */
export enum ETextureSurfaceShape {
  /** Flat and face on, where the decode is read most directly and tiling is judged. */
  PLANE = "plane",
  /** Curved, so the normal sweeps every grazing angle a wrong tangent sign shows up at. */
  SPHERE = "sphere",
  /** Edged, where a seam and the wrap of a tiling texture meet. */
  CUBE = "cube",
}

/** What each body is called, where a person chooses one. */
const SHAPE_LABELS: Record<ETextureSurfaceShape, string> = {
  [ETextureSurfaceShape.PLANE]: "Plane",
  [ETextureSurfaceShape.SPHERE]: "Sphere",
  [ETextureSurfaceShape.CUBE]: "Cube",
};

/**
 * Names one body.
 *
 * @param shape - Body to name.
 * @returns Its label.
 */
export function describeTextureSurfaceShape(shape: ETextureSurfaceShape): string {
  return SHAPE_LABELS[shape];
}

/**
 * How a texture's alpha channel is read, in the three answers the engine has for one.
 */
export enum ETextureSurfaceAlpha {
  /** Not read at all, which is what the plain deferred base shader does. */
  IGNORED = "ignored",
  /** Clipped against `def_aref`, which is what every `_aref` shader does. */
  CUT_OUT = "cut-out",
  /** Composited over what is behind it, which the deferred pass never does. */
  BLENDED = "blended",
}

/** What each answer is called, where a person chooses one. */
const ALPHA_LABELS: Record<ETextureSurfaceAlpha, string> = {
  [ETextureSurfaceAlpha.IGNORED]: "Ignored",
  [ETextureSurfaceAlpha.CUT_OUT]: "Cut out",
  [ETextureSurfaceAlpha.BLENDED]: "Blended",
};

/**
 * Names one way of reading alpha.
 *
 * @param alpha - The reading to name.
 * @returns Its label.
 */
export function describeTextureSurfaceAlpha(alpha: ETextureSurfaceAlpha): string {
  return ALPHA_LABELS[alpha];
}

/** How the surface is being looked at. */
export interface ITextureSurfaceOptions {
  shape: ETextureSurfaceShape;
  /** Whether the bump pair shades the surface, so the same body can be compared flat. */
  isBumped: boolean;
  /**
   * Whether a light shades the surface at all.
   *
   * Off, the body is drawn under a flat ambient and reads as the file does in the picture beside it. That comparison
   * is the reason both modes exist, and it only works when one of them adds nothing.
   */
  isLit: boolean;
  /** How many times the texture repeats across the body, which is how a tiling seam becomes visible. */
  tiling: number;
  /** Which of the engine's three readings of the alpha channel the body is drawn with. */
  alpha: ETextureSurfaceAlpha;
}

/**
 * The pair's texels on the cpu, for a pair whose layout stores them plainly.
 */
export interface ITextureBumpTexels {
  bump: IRenderTextureTexels;
  companion: IRenderTextureTexels;
}

/** The two located files behind a bump declaration, which are only ever fetched together. */
export interface ITextureBumpAssets {
  /** `normal.gloss`, the file the declaration names. */
  bump: XrayAsset;
  /** `normal_error.height`, the `#` companion. */
  companion: XrayAsset;
}

/** Nothing read, which is what a surface draws before a texture is chosen and after one is dropped. */
export const EMPTY_TEXTURE_SURFACE: ITextureSurfaceFiles = { aspect: 1, base: null, bump: null };

/**
 * The proportions of the base file, or a square when nothing measured it.
 *
 * @param description - The texture as the backend resolved it.
 * @returns Width over height.
 */
export function toTextureAspect(description: TextureDescription): number {
  const shape: Nullable<AssetTextureShape> = description.base?.shape ?? null;

  return shape && shape.height > 0 ? shape.width / shape.height : 1;
}

/**
 * The two files the engine binds for a texture, when it binds a pair with both halves located.
 *
 * Both halves or neither: the decode samples the pair every texel, and half of it shades nothing.
 *
 * @param description - The texture as the backend resolved it.
 * @returns The bump and its companion, or null when there is no pair to draw.
 */
export function selectTextureBumpAssets(description: TextureDescription): Nullable<ITextureBumpAssets> {
  // Null for a file outside a game tree as well as for one that declares no pair: nothing was resolved either way, and
  // a surface with nothing to bind draws the same in both cases.
  const bump = description.material?.bump ?? null;

  if (!bump) {
    return null;
  }

  const located: Nullable<XrayAsset> = getLocatedAsset(bump.bump.resolution);
  const companion: Nullable<XrayAsset> = getLocatedAsset(bump.companion.resolution);

  return located && companion ? { bump: located, companion } : null;
}
