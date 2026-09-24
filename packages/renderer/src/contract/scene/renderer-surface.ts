import { TRendererColor } from "#/contract/renderer-lighting";

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
  /** Added to what is under it, whole. */
  ADDED = "added",
  /** Added to what is under it, weighted by its own alpha. */
  ALPHA_ADDED = "alphaAdded",
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
  /** What the base is multiplied by, raw; white when left out. A viewer's affordance, not an engine term. */
  color?: TRendererColor;
  /** Whether a composited surface is lit, as a scripted pass may say it is not; lit when left out. */
  isLit?: boolean;
  /**
   * Whether it is a wall mark: composited into the albedo before any light, as the engine's wall mark phase does,
   * its texture sampled at the top level only. Its draw says how it composites there.
   */
  isWallmark?: boolean;
  /**
   * Whether it draws impostors, `details\lod`: the places of the object it dresses are impostors of a set, each a quad
   * blended from two facets and alpha tested as `lod.ps` does. Its `base` is the atlas, and its `hemi` the atlas's
   * `_nm` companion, a normal in colour and the hemisphere term in alpha.
   */
  isImpostor?: boolean;
}

/**
 * Which pass of the frame draws a surface, as the engine orders them.
 */
export enum ERendererPass {
  /** Into the G-buffer, lit by the deferred passes. */
  DEFERRED = "deferred",
  /** Into the G-buffer's albedo, before any light: the engine's wall mark phase. */
  WALLMARK = "wallmark",
  /** Composited over the tonemapped frame. */
  FORWARD = "forward",
}

/**
 * @param surface - What a consumer puts, or as much of it as decides this.
 * @returns The pass its draw puts it in.
 */
export function toRendererPass(surface: Pick<IRendererSurface, "draw" | "isWallmark">): ERendererPass {
  if (surface.draw === ERendererDraw.OPAQUE || surface.draw === ERendererDraw.CUT_OUT) {
    return ERendererPass.DEFERRED;
  }

  return surface.isWallmark ? ERendererPass.WALLMARK : ERendererPass.FORWARD;
}
