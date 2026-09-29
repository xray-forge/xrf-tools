/**
 * Where bolts strike and how far a strike lights the scene, as `CEffect_Thunderbolt` loads them: angles in radians.
 */
export interface IRendererThunderSettings {
  /** Above the horizon, the least and the most. */
  altitude: readonly [number, number];
  /** Either way of the heading opposite the sun. */
  deltaLongitude: number;
  /** The nearest a bolt strikes, of the far plane. */
  minDistance: number;
  /** The most a bolt leans off the vertical. */
  tilt: number;
  /** The chance a strike is followed at once by another. */
  secondProbability: number;
  /** How much of the strike's colour the sky, the sun and the fog are lit by. */
  skyColor: number;
  sunColor: number;
  fogColor: number;
}
