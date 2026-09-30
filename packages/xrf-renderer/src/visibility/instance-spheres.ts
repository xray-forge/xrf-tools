import { Matrix4, Sphere } from "three/webgpu";

import { RENDERER_FLOATS_PER_INSTANCE } from "#/contract/scene/renderer-instances";
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

  for (let index: number = 0; index < count; index += 1) {
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
 * @returns How many the view sees large enough to draw.
 */
export function collectVisibleInstances(view: CullView, spheres: Float32Array, into: Uint32Array): number {
  return collect(view, spheres, into, false);
}

/**
 * @param view - The view asking, which sees every instance whole.
 * @param spheres - Every instance's sphere, four floats each.
 * @param into - Where the indices of the instances large enough to draw are written, in their order.
 * @returns How many are, which only their size on screen decides.
 */
export function collectShownInstances(view: CullView, spheres: Float32Array, into: Uint32Array): number {
  return collect(view, spheres, into, true);
}

/** The instances kept, each tested for its size on screen and, unless the view sees them all, its visibility. */
function collect(view: CullView, spheres: Float32Array, into: Uint32Array, isInside: boolean): number {
  let count: number = 0;

  for (let at: number = 0, index: number = 0; at < spheres.length; index += 1, at += FLOATS_PER_SPHERE) {
    const x: number = spheres[at];
    const y: number = spheres[at + 1];
    const z: number = spheres[at + 2];
    const radius: number = spheres[at + 3];

    if (!view.isDiscarded(x, y, z, radius) && (isInside || view.classify(x, y, z, radius) !== EVisibility.OUTSIDE)) {
      into[count] = index;
      count += 1;
    }
  }

  return count;
}
