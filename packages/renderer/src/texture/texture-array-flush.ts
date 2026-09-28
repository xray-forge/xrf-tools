import { Texture } from "three/webgpu";

import { ITextureCopy } from "#/internals/texture-copy";

/** What an array asks the frame to do before it draws: copies, in order, then textures to let go once they are sent. */
export interface ITextureArrayFlush {
  copies: Array<ITextureCopy>;
  /** Arrays outgrown or emptied, disposed after the copies are sent. */
  disposals: Array<Texture>;
  /** Keys of arrays that were replaced, which bundles binding them record again for. */
  replaced: Array<string>;
  /** Keys copied into their layers, whose own textures can go once the copies are sent. */
  evicted: Array<string>;
}
