import { Texture } from "three/webgpu";

/** One copy between textures on the GPU: a level's whole blocks, of some layers, into layers at another place. */
export interface ITextureCopy {
  source: Texture;
  destination: Texture;
  level: number;
  /** Texels a side of the copy, whole blocks. */
  width: number;
  height: number;
  /** The first layer copied, and how many. */
  sourceLayer: number;
  layers: number;
  /** Where the first lands. */
  destinationLayer: number;
}
