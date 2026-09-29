/**
 * Where the sun stands at one hour of Monolith's sun table, in degrees, named as the table names them.
 */
export interface IRendererSunPosition {
  /** `sun_altitude`, which `setHP` makes the heading. */
  altitude: number;
  /** `sun_longitude`, which `setHP` makes the pitch. */
  longitude: number;
}
