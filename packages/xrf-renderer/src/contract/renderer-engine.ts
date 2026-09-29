/**
 * Which game the renderer draws as, named as the backend names its engine target: which rules its weather mixes by,
 * and which shaders it follows.
 */
export enum ERendererEngine {
  /** OpenXRay and the stock game: the keyframes stand the sun, or the astronomical sun does. */
  VANILLA = "vanilla",
  /**
   * Anomaly's Monolith: the sun table stands the sun, fog never reaches past the far plane, the sky is drawn through
   * the tonemap's curve, surfaces reflect the sky by the rain, and the rain wets them farther.
   */
  EXTENDED = "extended",
}
