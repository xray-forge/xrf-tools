import { TRendererColor } from "#/contract/renderer-color";
import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererSurfaceTextures } from "#/contract/scene/renderer-surface-textures";
import { IRendererSurfaceWater } from "#/contract/scene/renderer-surface-water";

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
  /** How a water surface is drawn; only a surface drawn as water reads it. */
  water?: IRendererSurfaceWater;
}
