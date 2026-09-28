import { StorageBufferAttribute, Texture } from "three/webgpu";

/** What the depth clip reads besides the frame. */
export interface IFsrClipInputs {
  /** The reconstructed depth of the frame before, a depth's bits a drawn texel. */
  reconstructed: StorageBufferAttribute;
  capacity: number;
  dilatedDepth: Texture;
  dilatedMotion: Texture;
  previousDilatedMotion: Texture;
  reactive: Texture;
}
