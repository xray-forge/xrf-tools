import { BufferGeometry } from "three/webgpu";

import { PACKED_TREE_COMPONENTS } from "#/geometry/renderer-packed-coordinate";
import { EVertexAttribute } from "#/geometry/vertex-attribute";

/**
 * @param geometry - A geometry.
 * @returns Whether its vertices are a tree's packed ones, whose coordinate is four shorts, two words (`v_tree`).
 */
export function isPackedTreeGeometry(geometry: BufferGeometry): boolean {
  return geometry.getAttribute(EVertexAttribute.PACKED_UV)?.itemSize === PACKED_TREE_COMPONENTS / 2;
}
