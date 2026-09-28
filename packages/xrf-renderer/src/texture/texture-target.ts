import { Texture } from "three/webgpu";

/** Whatever draws a key's texture: a sampler, or a slot a shared shader reads per material. */
export interface ITextureTarget {
  value: Texture;
}
