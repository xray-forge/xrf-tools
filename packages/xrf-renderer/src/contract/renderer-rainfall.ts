import { TRendererColor } from "#/contract/renderer-color";

/**
 * How hard it rains now, as the weather mixes it (`CEnvDescriptorMixer`).
 */
export interface IRendererRainfall {
  /** `rain_density`, from nought to one. */
  density: number;
  /** `rain_color`. */
  color: TRendererColor;
  /** `wind_velocity`, which leans the streaks. */
  windVelocity: number;
  /** `wind_direction`, in radians: the heading they lean to. */
  windDirection: number;
}
