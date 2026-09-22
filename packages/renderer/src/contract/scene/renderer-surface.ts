/**
 * How a surface reaches the frame, as the engine's blender for it decides.
 */
export enum ERendererDraw {
  /** Into the G-buffer, every texel. */
  OPAQUE = "opaque",
  /** Into the G-buffer, texels at or below the reference discarded. */
  CUT_OUT = "cutOut",
  /** Composited by alpha, after the deferred passes. */
  BLENDED = "blended",
  /** Added to what is under it. */
  ADDED = "added",
  /** Multiplied into what is under it. */
  MULTIPLIED = "multiplied",
  /** Multiplied at twice the strength, so mid grey leaves what is under it alone. */
  MULTIPLIED_2X = "multiplied2x",
  /** Submitted and drawn, writing nothing. */
  INVISIBLE = "invisible",
}

/**
 * The textures a surface samples, by the key each was put under.
 */
export interface IRendererSurfaceTextures {
  base?: string;
  detail?: string;
  /** The first half of a bump pair: the packed normal and its gloss. */
  bump?: string;
  /** The second half: the correction and the height. */
  bumpCompanion?: string;
  /** A baked lightmap: hemisphere occlusion in alpha, sun occlusion in green. */
  hemi?: string;
}

/**
 * One surface, as a level's shader table or a model's submesh describes it.
 */
export interface IRendererSurface {
  draw: ERendererDraw;
  /** The alpha a cut-out or blended texel must exceed, in `[0, 1]`. */
  alphaReference?: number;
  textures: IRendererSurfaceTextures;
  /** Detail texture repeats per base texture repeat. */
  detailScale?: number;
  /** The texture descriptor's lighting model: its class plus its weight. The engine's default is one. */
  material?: number;
  /** How many times the base and every other texture repeat across the surface. */
  tiling?: number;
}
