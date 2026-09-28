import { BufferAttribute, InterleavedBufferAttribute, WebGPURenderer } from "three/webgpu";

import { destroyGeometryAttribute } from "#/internals/renderer-backend";

/**
 * The buffers of geometries nothing draws any more, freed on the GPU at the next flush. Three frees a buffer only with
 * a geometry it drew, and the parts drawing a put geometry share its buffers, so the scene frees them itself.
 */
export class GeometryReleases {
  private released: Array<BufferAttribute | InterleavedBufferAttribute> = [];

  /**
   * @param attributes - Buffers no geometry drawn names any more.
   */
  public release(attributes: Iterable<BufferAttribute | InterleavedBufferAttribute>): void {
    for (const attribute of attributes) {
      this.released.push(attribute);
    }
  }

  /**
   * Frees every buffer released since the last flush.
   *
   * @param renderer - The renderer that uploaded them.
   */
  public free(renderer: WebGPURenderer): void {
    this.released.forEach((attribute: BufferAttribute | InterleavedBufferAttribute) =>
      destroyGeometryAttribute(renderer, attribute)
    );
    this.released = [];
  }
}
