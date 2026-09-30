import { float, select, vec4, vertexIndex } from "three/tsl";
import { Node } from "three/webgpu";

/**
 * @returns Where a triangle over the whole screen puts each corner, from its vertex index alone: `(-1, 3)`, `(-1, -1)`
 *   and `(3, -1)`, as three's `QuadMesh` places them, so the geometry is read for its count and its `uv` alone.
 */
export function toFullScreenVertex(): Node<"vec4"> {
  const far: Node<"float"> = float(3);
  const near: Node<"float"> = float(-1);

  return vec4(select(vertexIndex.equal(2), far, near), select(vertexIndex.equal(0), far, near), 0, 1);
}
