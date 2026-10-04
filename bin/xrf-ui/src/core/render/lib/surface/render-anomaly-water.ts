/**
 * Anomaly's own water (`shaders/r2/water.ps` in its gamedata), by the switches each of its programs defines: the whole
 * sky mixed with the base rather than the engine's share of it, a sun highlight, and foam only where asked.
 */
export interface IRenderAnomalyWater {
  /** `NEED_REFLECTIONS`: the sky mixed with the base by the base's alpha; without it, the base alone. */
  isReflecting: boolean;
  /** `NEED_SPECULARS`: the sun's Phong highlight, four times over. */
  isSpecular: boolean;
  /** `NEED_TRANSPARENT`: the base lit before it is mixed. */
  isTransparent: boolean;
  /** `NEED_FOAM`: foam in the shallows. */
  isFoamed: boolean;
}
