import { BufferAttribute } from "three/webgpu";

/** One copy between storage buffers on the GPU: the first bytes of one into the start of another. */
export interface IStorageCopy {
  source: BufferAttribute;
  destination: BufferAttribute;
  bytes: number;
}
