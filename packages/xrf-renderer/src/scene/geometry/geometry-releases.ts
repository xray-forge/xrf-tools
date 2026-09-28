import { BufferAttribute, BufferGeometry, InterleavedBufferAttribute, WebGPURenderer } from "three/webgpu";

import { disposeSharingGeometry } from "#/internals/geometry-disposal";
import { destroyGeometryAttribute } from "#/internals/renderer-backend";

/**
 * The buffers and geometries nothing draws any more, freed on the GPU at the next flush. Three frees a buffer only with
 * a geometry it drew, and the parts drawing a put geometry share its buffers, so the scene frees them itself.
 */
export class GeometryReleases {
  private released: Array<BufferAttribute | InterleavedBufferAttribute> = [];
  /** Geometries over another's buffers, each with the one it shares them with. */
  private sharing: Array<readonly [BufferGeometry, BufferGeometry]> = [];

  /**
   * @param attributes - Buffers no geometry drawn names any more.
   */
  public release(attributes: Iterable<BufferAttribute | InterleavedBufferAttribute>): void {
    for (const attribute of attributes) {
      this.released.push(attribute);
    }
  }

  /**
   * @param geometry - A geometry over another's buffers that nothing draws any more, its meshes disposed.
   * @param source - The geometry whose buffers it shares, which it lets go of none of.
   */
  public releaseSharing(geometry: BufferGeometry, source: BufferGeometry): void {
    this.sharing.push([geometry, source]);
  }

  /**
   * Frees every buffer and geometry released since the last flush.
   *
   * @param renderer - The renderer that uploaded them.
   */
  public free(renderer: WebGPURenderer): void {
    this.sharing.forEach(([geometry, source]: readonly [BufferGeometry, BufferGeometry]) =>
      disposeSharingGeometry(renderer, geometry, source)
    );
    this.released.forEach((attribute: BufferAttribute | InterleavedBufferAttribute) =>
      destroyGeometryAttribute(renderer, attribute)
    );
    this.sharing = [];
    this.released = [];
  }
}
