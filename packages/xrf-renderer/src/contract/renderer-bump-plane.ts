/**
 * One plane of a bump pair, as stored or as the engine reconstructs it.
 */
export enum ERendererBumpPlane {
  /** `normal.gloss` as stored: the normal reversed into green, blue and alpha, gloss in red. */
  BUMP = "bump",
  /** `normal_error.height` as stored: the quantisation error of the normal in rgb, height in alpha. */
  COMPANION = "companion",
  /** The tangent space normal the engine reconstructs, mapped into the unit range. */
  NORMAL = "normal",
  /** Gloss, the bump's red squared. */
  GLOSS = "gloss",
  /** Height as the companion stores it in alpha. */
  HEIGHT = "height",
}
