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
  index?: Uint16Array | Uint32Array;
  groups: ReadonlyArray<IRendererGeometryGroup>;
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
    geometry.index,
  ]
    .filter((array): array is NonNullable<typeof array> => array !== undefined)
    .map((array) => array.buffer);
}
