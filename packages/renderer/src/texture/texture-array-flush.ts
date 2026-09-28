import { Texture } from "three/webgpu";

import { ITextureCopy } from "#/internals/texture-copy";

/** What an array asks the frame to do before it draws: copies, in order, then textures to let go once they are sent. */
export interface ITextureArrayFlush {
  copies: Array<ITextureCopy>;
  /** Arrays it outgrew, and the keys' own textures no draw samples any more, disposed after the copies are sent. */
  disposals: Array<Texture>;
  /** Keys of arrays that were replaced, which bundles binding them record again for. */
  replaced: Array<string>;
  /** Keys whose own texture is among the disposals, which a bundle still sampling it records again for. */
  evicted: Array<string>;
}
