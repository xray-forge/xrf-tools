import { Box3, Sphere, Vector3, Vector4 } from "three/webgpu";

import { EVisibility } from "#/visibility/visibility";

/** A corner of a box, reused. */
const CORNER: Vector3 = new Vector3();

/**
 * @param box - A box, empty for one holding nothing.
 * @param planes - A volume's planes, normals pointing in, `w` the constant.
 * @returns Whether any of the box may be inside the volume: its corner furthest along each plane's normal is not behind
 *   it.
 */
export function isBoxInPlanes(box: Box3, planes: ReadonlyArray<Vector4>): boolean {
  if (box.isEmpty()) {
    return false;
  }

  return planes.every((plane: Vector4) => {
    CORNER.set(
      plane.x > 0 ? box.max.x : box.min.x,
      plane.y > 0 ? box.max.y : box.min.y,
      plane.z > 0 ? box.max.z : box.min.z
    );

    return plane.x * CORNER.x + plane.y * CORNER.y + plane.z * CORNER.z + plane.w >= 0;
  });
}

/**
 * @param x - The sphere's centre.
 * @param y - Its centre.
 * @param z - Its centre.
 * @param radius - Its radius.
 * @param planes - A volume's planes, normals pointing in, `w` the constant.
 * @returns How much of the sphere the volume holds: none once it is wholly behind a plane.
 */
export function toSphereVisibility(
  x: number,
  y: number,
  z: number,
  radius: number,
  planes: ReadonlyArray<Vector4>
): EVisibility {
  let visibility: EVisibility = EVisibility.INSIDE;

  for (const plane of planes) {
    const distance: number = plane.x * x + plane.y * y + plane.z * z + plane.w;

    if (distance < -radius) {
      return EVisibility.OUTSIDE;
    }

    if (distance < radius) {
      visibility = EVisibility.INTERSECTS;
    }
  }

  return visibility;
}

/**
 * @param sphere - A sphere.
 * @param planes - A volume's planes, normals pointing in, `w` the constant.
 * @returns Whether any of the sphere may be inside the volume.
 */
export function isSphereInPlanes(sphere: Sphere, planes: ReadonlyArray<Vector4>): boolean {
  const { center, radius } = sphere;

  return toSphereVisibility(center.x, center.y, center.z, radius, planes) !== EVisibility.OUTSIDE;
}
