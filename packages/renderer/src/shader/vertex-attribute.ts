/**
 * The vertex inputs the surface shaders read besides three's own, under the names geometry is built with.
 */
export enum EVertexAttribute {
  /** The authored binormal, which three has no name for. */
  BINORMAL = "binormal",
  /** A vertex's baked hemisphere term, the fourth byte of the engine's packed normal. */
  HEMI = "hemi",
  /** Its scale and offset, per place a geometry stands. */
  INSTANCE_HEMI = "instanceHemi",
}

/** The instanced columns a place's transform rides in, in order. */
export const INSTANCE_MATRIX_COLUMNS: ReadonlyArray<string> = [
  "instanceMatrix0",
  "instanceMatrix1",
  "instanceMatrix2",
  "instanceMatrix3",
];
