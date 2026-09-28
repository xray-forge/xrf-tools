import { TextureArray } from "#/texture/texture-array";

/** Where a key's texture is held: its array and its layer. */
export interface ITextureLayer {
  array: TextureArray;
  layer: number;
}
