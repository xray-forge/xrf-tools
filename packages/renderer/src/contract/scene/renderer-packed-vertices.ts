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
  /** Four bytes a vertex, `D3DCOLOR` as stored: the baked light blue, green, red, then the sun occlusion. */
  color?: Uint8Array;
  // Each coordinate starts on four bytes, as a sector's buffer lays its sections out: it is read as whole words.
}
