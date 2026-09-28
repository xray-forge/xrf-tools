import { BufferAttribute, WebGPURenderer } from "three/webgpu";

import { destroyStorageAttribute } from "#/internals/renderer-backend";

/**
 * Storage buffers let go of, freed a frame after, once no frame still being built can bind them: one queue for every
 * part that grows or drops storage, freed each frame whatever the frame draws.
 */
export class StorageRetirement {
  /** Retired before the frame now drawn, freed as the next begins. */
  private freeing: Array<BufferAttribute> = [];
  /** Retired since, held for the frame after. */
  private retired: Array<BufferAttribute> = [];

  /**
   * @param attributes - Buffers nothing binds any more.
   */
  public retire(attributes: Iterable<BufferAttribute>): void {
    for (const attribute of attributes) {
      this.retired.push(attribute);
    }
  }

  /**
   * Frees what was retired a frame ago and holds what was retired since. Once a frame.
   *
   * @param renderer - The renderer that uploaded them.
   */
  public free(renderer: WebGPURenderer): void {
    this.freeing.forEach((attribute: BufferAttribute) => destroyStorageAttribute(renderer, attribute));
    this.freeing = this.retired;
    this.retired = [];
  }
}
