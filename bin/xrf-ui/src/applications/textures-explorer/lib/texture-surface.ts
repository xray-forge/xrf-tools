import { Texture } from "three";

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

/** Nothing uploaded, which is what a surface draws before a texture is chosen and after one is dropped. */
export const EMPTY_TEXTURE_SURFACE: ITextureSurfaceTextures = { aspect: 1, base: null, bump: null };
