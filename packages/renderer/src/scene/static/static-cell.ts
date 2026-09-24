import { Nullable } from "@xrf/types";
import { Box3, Vector3, Vector4 } from "three/webgpu";

/** Metres a cell of the level is across, which a shadow cascade takes or leaves its casters by. */
export const STATIC_CELL_SIZE: number = 64;

/** The cell of what stands nowhere in particular: a draw whose extent is not known. */
export const STATIC_EVERYWHERE: string = "everywhere";

/** A box's centre, reused. */
const CENTER: Vector3 = new Vector3();

/**
 * @param bounds - What a static draw spans: a single draw's sphere's box, or the box of every place an instanced one
 *   stands in; null where it is not known.
 * @returns The cell it stands in, by its centre across the ground.
 */
export function toStaticCell(bounds: Nullable<Box3>): string {
  if (!bounds || bounds.isEmpty()) {
    return STATIC_EVERYWHERE;
  }

  bounds.getCenter(CENTER);

  return `${Math.floor(CENTER.x / STATIC_CELL_SIZE)},${Math.floor(CENTER.z / STATIC_CELL_SIZE)}`;
}

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
