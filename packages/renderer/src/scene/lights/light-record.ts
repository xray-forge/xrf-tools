/**
 * Where each vector of a light's record stands, four floats each, in view space: its position and `1 / L_R²`, its
 * colour and specular weight, its direction and `cos(cone / 2)`, its right and projection scale, its up and projector
 * slot, the sphere it is binned by, its shadow's near and far planes and face count, then a square of the atlas a face.
 */
export const LIGHT_RECORD = {
  position: 0,
  color: 1,
  axis: 2,
  right: 3,
  up: 4,
  sphere: 5,
  shadow: 6,
  faces: 7,
} as const;

/** Faces a light's shadow takes at most: a point's six. */
export const LIGHT_RECORD_FACES: number = 6;

/** Vectors of four floats a light's record takes. */
export const LIGHT_VECTORS: number = LIGHT_RECORD.faces + LIGHT_RECORD_FACES;
