import { BufferAttribute, BufferGeometry, Sphere, Vector3 } from "three/webgpu";

import { IRendererGeometry, IRendererGeometryGroup } from "#/contract/scene/renderer-geometry";
import { EVertexAttribute } from "#/shader/vertex-attribute";

/**
 * A geometry as three draws it, over the very arrays that crossed.
 *
 * @param geometry - What the consumer put.
 * @returns The geometry, bounded for culling, with a group per range.
 */
export function createRendererBufferGeometry(geometry: IRendererGeometry): BufferGeometry {
  const buffer: BufferGeometry = new BufferGeometry();

  buffer.setAttribute("position", new BufferAttribute(geometry.position, 3));

  if (geometry.uv) {
    buffer.setAttribute("uv", new BufferAttribute(geometry.uv, 2));
  }

  if (geometry.uv1) {
    buffer.setAttribute("uv1", new BufferAttribute(geometry.uv1, 2));
  }

  if (geometry.tangent) {
    buffer.setAttribute("tangent", new BufferAttribute(geometry.tangent, 3));
  }

  if (geometry.binormal) {
    buffer.setAttribute(EVertexAttribute.BINORMAL, new BufferAttribute(geometry.binormal, 3));
  }

  if (geometry.skinIndices && geometry.skinWeights) {
    buffer.setAttribute("skinIndex", new BufferAttribute(geometry.skinIndices, 4));
    buffer.setAttribute("skinWeight", new BufferAttribute(geometry.skinWeights, 4));
  }

  if (geometry.hemi) {
    buffer.setAttribute(EVertexAttribute.HEMI, new BufferAttribute(geometry.hemi, 1));
  }

  if (geometry.index) {
    buffer.setIndex(new BufferAttribute(geometry.index, 1));
  }

  if (geometry.normal) {
    buffer.setAttribute("normal", new BufferAttribute(geometry.normal, 3));
  } else {
    // The G-buffer stores a normal for every pixel; a geometry without one is given the one its faces imply.
    buffer.computeVertexNormals();
  }

  if (geometry.groups.length) {
    geometry.groups.forEach(({ start, count, slot }: IRendererGeometryGroup) => buffer.addGroup(start, count, slot));
  } else {
    buffer.addGroup(0, geometry.index ? geometry.index.length : geometry.position.length / 3, 0);
  }

  if (geometry.bounds) {
    buffer.boundingSphere = new Sphere(new Vector3(...geometry.bounds.center), geometry.bounds.radius);
  } else {
    buffer.computeBoundingSphere();
  }

  return buffer;
}
