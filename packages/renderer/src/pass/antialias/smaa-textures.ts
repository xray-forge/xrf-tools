import { Texture } from "three/webgpu";

/** What SMAA's stages sample, as textures the pass holds. */
export interface ISmaaTextures {
  frame: Texture;
  edges: Texture;
  weights: Texture;
  area: Texture;
  search: Texture;
}
