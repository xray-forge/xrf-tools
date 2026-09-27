import { Box3, Sphere, Vector3, Vector4 } from "three/webgpu";

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
 * @param sphere - A sphere.
 * @param planes - A volume's planes, normals pointing in, `w` the constant.
 * @returns Whether any of the sphere may be inside the volume: its centre no further behind any plane than its radius.
 */
export function isSphereInPlanes(sphere: Sphere, planes: ReadonlyArray<Vector4>): boolean {
  const { center, radius } = sphere;

  return planes.every(
    (plane: Vector4) => plane.x * center.x + plane.y * center.y + plane.z * center.z + plane.w >= -radius
  );
}
