/**
 * Which engine a weather is mixed as, named as the backend names its engine target.
 */
export enum ERendererWeatherEngine {
  /** OpenXRay and the stock game: the keyframes stand the sun, or the astronomical sun does. */
  VANILLA = "vanilla",
  /** Anomaly's Monolith: the sun table stands the sun, and fog never reaches past the far plane. */
  EXTENDED = "extended",
}
