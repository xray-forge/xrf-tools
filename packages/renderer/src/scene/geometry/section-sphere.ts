import { Nullable } from "@xrf/types";
import { Box3, Sphere, Vector3 } from "three/webgpu";

/**
 * The sphere around the vertices a range of a geometry reaches: centred on their box, as far out as the farthest.
 *
 * @param positions - Three floats a vertex.
 * @param index - The geometry's indices, or null for one drawn in vertex order.
 * @param start - The range's first index, or first vertex without indices.
 * @param count - Indices, or vertices, in it.
 * @returns The sphere, empty for a range reaching nothing.
 */
export function toSectionSphere(
  positions: ArrayLike<number>,
  index: Nullable<ArrayLike<number>>,
  start: number,
  count: number
): Sphere {
  const box: Box3 = new Box3();
  const point: Vector3 = new Vector3();

  function vertexAt(at: number): number {
    return (index ? index[at] : at) * 3;
  }

  for (let at = start; at < start + count; at += 1) {
    const vertex: number = vertexAt(at);

    box.expandByPoint(point.set(positions[vertex], positions[vertex + 1], positions[vertex + 2]));
  }

  const sphere: Sphere = new Sphere();

  if (box.isEmpty()) {
    return sphere.makeEmpty();
  }

  box.getCenter(sphere.center);

  let farthest: number = 0;

  for (let at = start; at < start + count; at += 1) {
    const vertex: number = vertexAt(at);

    farthest = Math.max(
      farthest,
      sphere.center.distanceToSquared(point.set(positions[vertex], positions[vertex + 1], positions[vertex + 2]))
    );
  }

  sphere.radius = Math.sqrt(farthest);

  return sphere;
}
