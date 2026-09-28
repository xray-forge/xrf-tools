/**
 * The vertex inputs the surface shaders read besides three's own, under the names geometry is built with.
 */
export enum EVertexAttribute {
  /** The authored binormal, which three has no name for. */
  BINORMAL = "binormal",
  /** A vertex's baked hemisphere term, the fourth byte of the engine's packed normal. */
  HEMI = "hemi",
  /** The engine's packed normal, four bytes with the hemisphere term last, in place of three's float `normal`. */
  PACKED_NORMAL = "packedNormal",
  /** The engine's packed tangent, the base `u`'s low byte last. */
  PACKED_TANGENT = "packedTangent",
  /** The engine's packed binormal, the base `v`'s low byte last. */
  PACKED_BINORMAL = "packedBinormal",
  /** The base coordinate as shorts, two a vertex or a tree's four, in place of three's `uv`. */
  PACKED_UV = "packedUv",
  /** The lightmap coordinate as shorts, in place of three's `uv1`. */
  PACKED_UV1 = "packedUv1",
  /**
   * The engine's vertex colour, its baked light and its sun occlusion, which water is lit by: drawn plainly, so no
   * arena stores it.
   */
  PACKED_COLOR = "packedColor",
  /** Its scale and offset, per place a geometry stands. */
  INSTANCE_HEMI = "instanceHemi",
  /**
   * The mark of a clustered static draw's arena, followed by the arena's number: never read, only named, so the
   * programs built for one arena's draws, which read its buffers by name, are its own.
   */
  CLUSTER_ARENA = "clusterArena",
}

/** The instanced columns a place's transform rides in, in order. */
export const INSTANCE_MATRIX_COLUMNS: ReadonlyArray<string> = [
  "instanceMatrix0",
  "instanceMatrix1",
  "instanceMatrix2",
  "instanceMatrix3",
];
