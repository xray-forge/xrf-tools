import { BufferAttribute, WebGPURenderer } from "three/webgpu";

import { destroyStorageAttribute } from "#/internals/renderer-backend";
import { releaseStorageArray } from "#/internals/storage-buffers";
import { markZeroedStorage } from "#/internals/zeroed-storage";

/**
 * Storage buffers let go of, freed a frame after, once no frame still being built can bind them: one queue for every
 * part that grows or drops storage, freed each frame whatever the frame draws. It also lets go of the CPU arrays of
 * buffers only ever uploaded once, or written on the GPU alone, as soon as three holds them there.
 */
export class StorageRetirement {
  /** Retired before the frame now drawn, freed as the next begins. */
  private freeing: Array<BufferAttribute> = [];
  /** Retired since, held for the frame after. */
  private retired: Array<BufferAttribute> = [];
  /** Buffers whose CPU arrays go once three made them on the GPU. */
  private readonly uploading: Set<BufferAttribute> = new Set();

  /**
   * @param attributes - Buffers nothing binds any more.
   */
  public retire(attributes: Iterable<BufferAttribute>): void {
    for (const attribute of attributes) {
      this.retired.push(attribute);
      this.uploading.delete(attribute);
    }
  }

  /**
   * @param attributes - Buffers the CPU never writes or reads after making them: their arrays only exist for three to
   *   make them from, and go once it has.
   */
  public retireArrays(attributes: Iterable<BufferAttribute>): void {
    for (const attribute of attributes) {
      this.uploading.add(attribute);
    }
  }

  /**
   * @param attributes - Buffers written on the GPU alone and zero until then: made on the device at their arrays' size
   *   with nothing sent, and their arrays let go as three holds them.
   */
  public retireZeroed(attributes: ReadonlyArray<BufferAttribute>): void {
    markZeroedStorage(attributes);
    this.retireArrays(attributes);
  }

  /**
   * Frees what was retired a frame ago and holds what was retired since, and lets go of the arrays of buffers three
   * made meanwhile. Once a frame.
   *
   * @param renderer - The renderer that uploaded them.
   */
  public free(renderer: WebGPURenderer): void {
    this.freeing.forEach((attribute: BufferAttribute) => destroyStorageAttribute(renderer, attribute));
    this.freeing = this.retired;
    this.retired = [];

    for (const attribute of this.uploading) {
      if (releaseStorageArray(renderer, attribute)) {
        this.uploading.delete(attribute);
      }
    }
  }
}
