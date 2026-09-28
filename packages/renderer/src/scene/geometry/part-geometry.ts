import { BufferAttribute, BufferGeometry, InstancedBufferGeometry, InterleavedBufferAttribute } from "three/webgpu";

import { ISceneSection } from "#/scene/geometry/scene-section";

/**
 * A geometry drawing one section of another: the same buffers, uploaded once for every part sharing them, drawn over
 * the section's own range and bounded by its own sphere. It owns none of them: `disposeSharingGeometry` lets it go.
 *
 * @param source - The geometry whose buffers it draws, instanced or not.
 * @param section - The range it draws.
 * @returns The part.
 */
export function createPartGeometry<T extends BufferGeometry>(source: T, section: ISceneSection): T {
  const part = (source instanceof InstancedBufferGeometry ? new InstancedBufferGeometry() : new BufferGeometry()) as T;

  part.index = source.index;
  Object.entries(source.attributes).forEach(
    ([name, attribute]: [string, BufferAttribute | InterleavedBufferAttribute]) => part.setAttribute(name, attribute)
  );
  part.setDrawRange(section.start, section.count);
  // Set rather than measured: measuring walks every position of the source, once for each part.
  part.boundingSphere = section.sphere;

  if (part instanceof InstancedBufferGeometry && source instanceof InstancedBufferGeometry) {
    part.instanceCount = source.instanceCount;
  }

  return part;
}

/**
 * Disposes a geometry drawing another's buffers without them: three frees every buffer a disposed geometry names, and
 * these are the source's, drawn by every other geometry sharing them.
 *
 * @param geometry - A part, or another geometry built over the source's buffers.
 * @param source - The geometry whose buffers it shares.
 */
export function disposeSharingGeometry(geometry: BufferGeometry, source: BufferGeometry): void {
  if (geometry.index === source.index) {
    geometry.setIndex(null);
  }

  Object.entries(source.attributes).forEach(
    ([name, attribute]: [string, BufferAttribute | InterleavedBufferAttribute]) => {
      if (geometry.getAttribute(name) === attribute) {
        geometry.deleteAttribute(name);
      }
    }
  );

  geometry.dispose();
}
