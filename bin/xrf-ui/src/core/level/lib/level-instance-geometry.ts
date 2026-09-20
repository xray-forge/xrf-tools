import { BufferGeometry, InstancedMesh, Material, Matrix4 } from "three";

import { ISectorInstanceViews } from "@/core/level/lib/level-sector-views";

/** Floats one instance's transform occupies, which is a whole four by four matrix. */
export const FLOATS_PER_INSTANCE: number = 16;

/**
 * Builds the mesh that stands one geometry in every place the level puts it.
 *
 * @param group - The mesh, its attributes and its transforms.
 * @param geometry - Geometry built for it.
 * @param material - Material the surface is drawn with.
 * @returns An instanced mesh with every place already written into it.
 */
export function createInstancedMesh(
  group: ISectorInstanceViews,
  geometry: BufferGeometry,
  material: Material
): InstancedMesh {
  const mesh: InstancedMesh = new InstancedMesh(geometry, material, group.instanceCount);
  const matrix: Matrix4 = new Matrix4();

  for (let index = 0; index < group.instanceCount; index += 1) {
    // The engine stores a row-vector matrix row major; three.js reads a column-vector one column major. Those two are
    // transposes of each other, so the same sixteen floats mean the same transform and nothing is rearranged.
    matrix.fromArray(group.transforms, index * FLOATS_PER_INSTANCE);
    mesh.setMatrixAt(index, matrix);
  }

  mesh.instanceMatrix.needsUpdate = true;
  // Over the places rather than over the mesh: instances stand by their own matrices, so a sphere around the mesh's
  // own origin would cull a whole stand the moment that origin left the view. `InstancedMesh` measures the instance
  // matrices, which is the extent a frustum test actually wants - and computed here, where the matrices have just
  // been written, rather than lazily inside the first frame that tests it.
  mesh.computeBoundingSphere();

  return mesh;
}
