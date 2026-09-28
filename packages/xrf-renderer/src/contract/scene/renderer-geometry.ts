import { Maybe } from "@xrf/types";

import { IRendererBounds } from "#/contract/scene/renderer-bounds";
import { IRendererClusters } from "#/contract/scene/renderer-clusters";
import { IRendererGeometryGroup } from "#/contract/scene/renderer-geometry-group";
import { IRendererPackedVertices } from "#/contract/scene/renderer-packed-vertices";

/**
 * Vertices and indices in renderer space, as flat arrays handed over rather than copied.
 * An attribute left out is one the geometry does not have; a surface sampling it samples a neutral value.
 */
export interface IRendererGeometry {
  /** Three floats a vertex. */
  position: Float32Array;
  /** Three floats a vertex. */
  normal?: Float32Array;
  /** Two floats a vertex: the base and detail coordinates. */
  uv?: Float32Array;
  /** Two floats a vertex: the coordinates a baked lightmap is sampled at. */
  uv1?: Float32Array;
  /** Three floats a vertex: the authored tangent, which a bump pair's x rotates along. */
  tangent?: Float32Array;
  /** Three floats a vertex: the authored binormal, kept rather than rebuilt so a skewed basis stays as authored. */
  binormal?: Float32Array;
  /** Four bone indices a vertex, for an object skinned to a skeleton. */
  skinIndices?: Uint16Array;
  /** Four weights a vertex, beside the indices. */
  skinWeights?: Float32Array;
  /** One float a vertex: the hemisphere occlusion a vertex-lit surface carries in its normal's fourth byte. */
  hemi?: Float32Array;
  /**
   * The normal, tangent, binormal, coordinates and hemisphere term as the engine packs them, in place of the float
   * ones: a geometry carries one form or the other, never both.
   */
  packed?: IRendererPackedVertices;
  index?: Uint16Array | Uint32Array;
  groups: ReadonlyArray<IRendererGeometryGroup>;
  /** Its clusters, which a range must be cut into to be drawn as a static draw; drawn plainly without. */
  clusters?: IRendererClusters;
  /** What the positions span, where it was measured already; measured by the renderer otherwise. */
  bounds?: IRendererBounds;
}

/**
 * What of a geometry moves between threads: the arrays, never copied.
 *
 * @param geometry - The geometry about to be posted.
 * @returns Its buffers.
 */
export function listRendererGeometryTransfers(geometry: IRendererGeometry): Array<Transferable> {
  return [
    geometry.position,
    geometry.normal,
    geometry.uv,
    geometry.uv1,
    geometry.tangent,
    geometry.binormal,
    geometry.skinIndices,
    geometry.skinWeights,
    geometry.hemi,
    geometry.packed?.normal,
    geometry.packed?.tangent,
    geometry.packed?.binormal,
    geometry.packed?.uv,
    geometry.packed?.uv1,
    geometry.packed?.color,
    geometry.index,
    geometry.clusters?.ranges,
    geometry.clusters?.spheres,
  ].flatMap((array: Maybe<ArrayBufferView>) => (array ? [array.buffer] : []));
}
