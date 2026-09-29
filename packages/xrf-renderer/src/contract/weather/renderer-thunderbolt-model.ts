import { ERendererDraw } from "#/contract/scene/renderer-draw";

/**
 * A bolt's `lightning_model` (`dm\*.dm`): its mesh in engine space, from its top down its length of one, and its
 * texture.
 */
export interface IRendererThunderboltModel {
  /** The texture's reference, a key of the weather's textures. */
  texture: string;
  /** How its shader composites it. */
  draw: ERendererDraw;
  /** Three numbers a vertex. */
  positions: ReadonlyArray<number>;
  /** Two numbers a vertex. */
  uvs: ReadonlyArray<number>;
  /** A triangle list. */
  indices: ReadonlyArray<number>;
}
