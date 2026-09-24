import { TRendererVector } from "#/contract/renderer-lighting";

/**
 * A range of a geometry's indices drawn with one surface.
 */
export interface IRendererGeometryGroup {
  /** First index of the range. */
  start: number;
  /** Indices in it. */
  count: number;
  /** Which of the object's surfaces draws it, by position in its list. */
  slot: number;
  /** What the range's own vertices span, which it is culled by; measured by the renderer where left out. */
  bounds?: IRendererBounds;
}

/**
 * A sphere enclosing a geometry's positions, as whoever made them measured it.
 */
export interface IRendererBounds {
  center: TRendererVector;
  radius: number;
}

/**
 * A level's vertex attributes as xrLC packed them, 20 bytes a vertex beside a float position where the same vertex
 * decoded takes 32. A direction is four `D3DCOLOR` bytes, its z, y and x as `(d + 1) * 127.5` in renderer space, and
 * the fourth byte carries something else: the hemisphere term on the normal, the low byte of the base coordinate's
 * `u` on the tangent and of its `v` on the binormal.
 */
export interface IRendererPackedVertices {
  /** Four bytes a vertex: the normal, then the hemisphere term. */
  normal: Uint8Array;
  /** Four bytes a vertex: the authored tangent, then the low byte of the base `u`; zeros where none was authored. */
  tangent?: Uint8Array;
  /** Four bytes a vertex: the authored binormal, then the low byte of the base `v`. */
  binormal?: Uint8Array;
  /**
   * The base coordinate as xrLC quantised it: two shorts a vertex (`SHORT2`), over 1024 once the low bytes are added
   * (`unpack_tc_base`); or four (`SHORT4`), a tree's, over 2048 with no low bytes, then its wind terms.
   */
  uv?: Int16Array;
  /** The lightmap coordinate: two shorts a vertex, over 32768 (`unpack_tc_lmap`). */
  uv1?: Int16Array;
  // Each coordinate starts on four bytes, as a sector's buffer lays its sections out: it is read as whole words.
}

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
    geometry.index,
  ]
    .filter((array): array is NonNullable<typeof array> => array !== undefined)
    .map((array) => array.buffer);
}
