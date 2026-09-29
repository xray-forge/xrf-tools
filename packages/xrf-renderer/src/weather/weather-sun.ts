import { IRendererSunPosition } from "#/contract/weather/renderer-sun-position";

/**
 * Where a mix stands the sun.
 */
export enum EWeatherSun {
  /** Lerped between the two keyframes' own directions: OpenXRay with its dynamic sun off. */
  AUTHORED = "authored",
  /** OpenXRay's astronomical sun, `calculate_dynamic_sun_dir`. */
  DYNAMIC = "dynamic",
  /** Monolith's hourly table, `calculate_config_sun_dir`. */
  TABLE = "table",
}

export type TWeatherSun =
  | { kind: EWeatherSun.AUTHORED }
  | { kind: EWeatherSun.DYNAMIC }
  | { kind: EWeatherSun.TABLE; positions: ReadonlyArray<IRendererSunPosition> };
