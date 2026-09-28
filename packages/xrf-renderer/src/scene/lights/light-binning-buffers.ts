import { StorageBufferAttribute } from "three/webgpu";

/** What the lights are binned from and into. */
export interface ILightBinningBuffers {
  readonly records: StorageBufferAttribute;
  readonly counts: StorageBufferAttribute;
  readonly items: StorageBufferAttribute;
  /** Lights each cluster was reached by and could not hold. */
  readonly drops: StorageBufferAttribute;
}
