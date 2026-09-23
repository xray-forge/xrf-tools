import { Matrix4, Sphere } from "three/webgpu";

import { RENDERER_FLOATS_PER_INSTANCE } from "#/contract/scene/renderer-object";
import { CullView } from "#/visibility/cull-view";
import { EVisibility } from "#/visibility/visibility";

/** Floats one instance's sphere takes: its centre, then its radius. */
export const FLOATS_PER_SPHERE: number = 4;

/**
 * Every instance's sphere in renderer space: the mesh's own, stood in each place.
 *
 * @param sphere - The mesh's sphere, in its own space.
 * @param transforms - Sixteen floats an instance, column major.
 * @param placement - What places every instance, the object's own matrix.
 * @returns Four floats an instance.
 */
export function toInstanceSpheres(sphere: Sphere, transforms: Float32Array, placement: Matrix4): Float32Array {
  const count: number = transforms.length / RENDERER_FLOATS_PER_INSTANCE;
  const spheres: Float32Array = new Float32Array(count * FLOATS_PER_SPHERE);
  const matrix: Matrix4 = new Matrix4();
  const placed: Sphere = new Sphere();

  for (let index = 0; index < count; index += 1) {
    const at: number = index * FLOATS_PER_SPHERE;

    matrix.fromArray(transforms, index * RENDERER_FLOATS_PER_INSTANCE).premultiply(placement);
    placed.copy(sphere).applyMatrix4(matrix);
    spheres[at] = placed.center.x;
    spheres[at + 1] = placed.center.y;
    spheres[at + 2] = placed.center.z;
    spheres[at + 3] = placed.radius;
  }

  return spheres;
}

/**
 * @param view - The view asking.
 * @param spheres - Every instance's sphere, four floats each.
 * @param into - Where the indices of the instances seen are written, in their order.
 * @returns How many the view sees.
 */
export function collectVisibleInstances(view: CullView, spheres: Float32Array, into: Uint32Array): number {
  let count: number = 0;

  for (let at = 0, index = 0; at < spheres.length; index += 1, at += FLOATS_PER_SPHERE) {
    if (view.classify(spheres[at], spheres[at + 1], spheres[at + 2], spheres[at + 3]) !== EVisibility.OUTSIDE) {
      into[count] = index;
      count += 1;
    }
  }

  return count;
}
