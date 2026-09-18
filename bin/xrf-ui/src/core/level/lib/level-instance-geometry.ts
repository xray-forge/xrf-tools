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
  // Instances are placed by their own matrices rather than by the mesh's, so a frustum test against its unmoved
  // bounding sphere would cull the whole stand the moment the mesh's own origin left the view.
  mesh.frustumCulled = false;

  return mesh;
}
