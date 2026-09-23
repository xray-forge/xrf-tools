import { Nullable } from "@xrf/types";
import { BufferGeometry, Material, Matrix4, Mesh, Skeleton, SkinnedMesh } from "three/webgpu";

/**
 * A mesh the scene places itself: its matrix set directly, never recomposed, and never culled by three, since the
 * scene's own culling decides what draws.
 *
 * @param geometry - What it draws.
 * @param skeleton - What it is skinned to, or null for a rigid mesh.
 * @param material - What draws it, until it is shown with another.
 * @returns The mesh.
 */
export function createSceneMesh(geometry: BufferGeometry, skeleton: Nullable<Skeleton>, material?: Material): Mesh {
  const mesh: Mesh = skeleton ? new SkinnedMesh(geometry, material) : new Mesh(geometry, material);

  mesh.matrixAutoUpdate = false;
  mesh.frustumCulled = false;

  if (mesh instanceof SkinnedMesh && skeleton) {
    // Identity: the vertices and the bone transforms are both in model space already.
    mesh.bind(skeleton, new Matrix4());
  }

  return mesh;
}
