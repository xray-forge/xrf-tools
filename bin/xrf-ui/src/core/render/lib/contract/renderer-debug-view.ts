/**
 * Which picture reaches the canvas: the frame, or one of the targets it was built from.
 */
export enum ERendererDebugView {
  /** The finished frame. */
  FINAL = "final",
  /** The G-buffer's albedo, raw. */
  ALBEDO = "albedo",
  /** The gloss the albedo target carries in its alpha. */
  GLOSS = "gloss",
  /** The view space normal, remapped to colour. */
  NORMAL = "normal",
  /** The baked hemisphere occlusion. */
  HEMI = "hemi",
  /** The baked sun occlusion. */
  SUN = "sun",
  /** The lighting model slice, `(class + 0.5) / 4`. */
  MATERIAL = "material",
  /** View depth, near dark and far light. */
  DEPTH = "depth",
  /** What the lights accumulated: diffuse in colour. */
  LIGHT = "light",
  /** The screen's occlusion, white where it is off. */
  AMBIENT_OCCLUSION = "ambient-occlusion",
  /** How far each surface moved on the screen since the frame before: grey for none, eight pixels to a full shift. */
  MOTION = "motion",
}
