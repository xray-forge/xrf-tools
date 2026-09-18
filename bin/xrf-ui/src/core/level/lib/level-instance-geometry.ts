import { BufferAttribute, BufferGeometry, InstancedMesh, Material, Matrix4 } from "three";

import { HEMI_ATTRIBUTE, LIGHTMAP_ATTRIBUTE } from "@/core/level/lib/level-sector-geometry";
import { ISectorInstanceViews } from "@/core/level/lib/level-sector-views";

/** Floats one instance's transform occupies, which is a whole four by four matrix. */
export const FLOATS_PER_INSTANCE: number = 16;

/**
 * Builds the geometry of one instanced mesh.
 *
 * @param group - The mesh and its attribute views.
 * @returns Geometry for an instanced draw.
 */
export function createInstanceGeometry(group: ISectorInstanceViews): BufferGeometry {
  const geometry: BufferGeometry = new BufferGeometry();

  geometry.setAttribute("position", new BufferAttribute(group.positions, 3));
  geometry.setIndex(new BufferAttribute(group.indices, 1));

  if (group.normals) {
    geometry.setAttribute("normal", new BufferAttribute(group.normals, 3));
  }

  if (group.uvs) {
    geometry.setAttribute("uv", new BufferAttribute(group.uvs, 2));
  }

  if (group.lightmapUvs) {
    geometry.setAttribute(LIGHTMAP_ATTRIBUTE, new BufferAttribute(group.lightmapUvs, 2));
  }

  if (group.colors) {
    geometry.setAttribute("color", new BufferAttribute(group.colors, 3));
  }

  if (group.hemi) {
    geometry.setAttribute(HEMI_ATTRIBUTE, new BufferAttribute(group.hemi, 1));
  }

  geometry.computeBoundingSphere();

  return geometry;
}

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
    // The engine stores a row-vector matrix row major; three.js reads a column-vector one column major. Those two
    // are transposes of each other, so the same sixteen floats mean the same transform and nothing is rearranged.
    matrix.fromArray(group.transforms, index * FLOATS_PER_INSTANCE);
    mesh.setMatrixAt(index, matrix);
  }

  mesh.instanceMatrix.needsUpdate = true;
  mesh.frustumCulled = false;

  return mesh;
}
