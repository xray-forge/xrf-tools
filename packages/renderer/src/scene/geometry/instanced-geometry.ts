import {
  BufferGeometry,
  InstancedBufferAttribute,
  InstancedBufferGeometry,
  InstancedInterleavedBuffer,
  InterleavedBufferAttribute,
  Matrix4,
  Sphere,
} from "three/webgpu";

import { IRendererInstances } from "#/contract/scene/renderer-object";
import { EVertexAttribute, INSTANCE_MATRIX_COLUMNS } from "#/shader/vertex-attribute";

/** Floats one instance's transform takes. */
const FLOATS_PER_INSTANCE: number = 16;

/**
 * A geometry standing in every place the instances name: the put geometry's own attributes, the places as instanced
 * columns, and a sphere around all of them for culling.
 *
 * @param base - The geometry put, whose attributes are shared rather than copied.
 * @param instances - Where it stands.
 * @returns The geometry.
 */
export function createInstancedGeometry(base: BufferGeometry, instances: IRendererInstances): InstancedBufferGeometry {
  const geometry: InstancedBufferGeometry = new InstancedBufferGeometry();
  const columns: InstancedInterleavedBuffer = new InstancedInterleavedBuffer(instances.transforms, FLOATS_PER_INSTANCE);
  const count: number = instances.transforms.length / FLOATS_PER_INSTANCE;

  geometry.index = base.index;
  Object.entries(base.attributes).forEach(([name, attribute]) => geometry.setAttribute(name, attribute));
  base.groups.forEach((group) => geometry.addGroup(group.start, group.count, group.materialIndex));

  INSTANCE_MATRIX_COLUMNS.forEach((column: string, index: number) =>
    geometry.setAttribute(column, new InterleavedBufferAttribute(columns, 4, index * 4))
  );

  if (instances.hemi) {
    geometry.setAttribute(EVertexAttribute.INSTANCE_HEMI, new InstancedBufferAttribute(instances.hemi, 2));
  }

  geometry.instanceCount = count;
  geometry.boundingSphere = toInstancesSphere(base, instances.transforms, count);

  return geometry;
}

/** A sphere around the base's own, stood in every place. */
function toInstancesSphere(base: BufferGeometry, transforms: Float32Array, count: number): Sphere {
  if (!base.boundingSphere) {
    base.computeBoundingSphere();
  }

  const own: Sphere = base.boundingSphere as Sphere;
  const matrix: Matrix4 = new Matrix4();
  const placed: Sphere = new Sphere();
  const all: Sphere = new Sphere();

  for (let index = 0; index < count; index += 1) {
    placed.copy(own).applyMatrix4(matrix.fromArray(transforms, index * FLOATS_PER_INSTANCE));

    if (index === 0) {
      all.copy(placed);
    } else {
      all.union(placed);
    }
  }

  return all;
}
