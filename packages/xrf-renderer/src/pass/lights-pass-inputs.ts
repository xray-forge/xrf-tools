import { Data3DTexture, StorageBufferAttribute, Texture, TextureNode } from "three/webgpu";

import { IGBufferTextures } from "#/shader/gbuffer-textures";

/** What the lights are accumulated from. */
export interface ILightsPassInputs {
  gbuffer: IGBufferTextures;
  records: StorageBufferAttribute;
  counts: StorageBufferAttribute;
  items: StorageBufferAttribute;
  lut: Data3DTexture;
  /** A sampler a projector slot. */
  projectors: ReadonlyArray<TextureNode>;
  /** The shadow atlas's depth, reversed. */
  atlas: Texture;
}
