import { Texture } from "three";

import { getLocatedAsset } from "@/core/assets/lib";
import { AssetTextureShape, TextureDescription } from "@/core/bindings/types/xrf-app";
import { XrayAsset } from "@/core/bindings/types/xrf-vfs";
import { IVisualBumpTextures } from "@/core/visuals/lib/visual-bump";
import { Nullable } from "@/lib/types/general";

/**
 * What one texture is drawn from once its files are on the gpu.
 */
export interface ITextureSurfaceTextures {
  /** The base texture, or null for a descriptor with no texture beside it. */
  base: Nullable<Texture>;
  /** The pair the engine binds, or null for a material that binds none. */
  bump: Nullable<IVisualBumpTextures>;
  /**
   * Width over height of the base file, so a flat body is drawn in the proportions the texture was authored in.
   */
  aspect: number;
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
}

/** The two located files behind a bump declaration, which are only ever fetched together. */
export interface ITextureBumpAssets {
  /** `normal.gloss`, the file the declaration names. */
  bump: XrayAsset;
  /** `normal_error.height`, the `#` companion. */
  companion: XrayAsset;
}

/** Nothing uploaded, which is what a surface draws before a texture is chosen and after one is dropped. */
export const EMPTY_TEXTURE_SURFACE: ITextureSurfaceTextures = { aspect: 1, base: null, bump: null };

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
  const { bump } = description.material;

  if (!bump) {
    return null;
  }

  const located: Nullable<XrayAsset> = getLocatedAsset(bump.bump.resolution);
  const companion: Nullable<XrayAsset> = getLocatedAsset(bump.companion.resolution);

  return located && companion ? { bump: located, companion } : null;
}
