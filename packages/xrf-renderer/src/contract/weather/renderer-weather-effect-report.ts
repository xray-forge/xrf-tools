/**
 * The weather effect the renderer plays over its cycle.
 */
export interface IRendererWeatherEffectReport {
  name: string;
  /** Game seconds until the cycle takes over again. */
  remaining: number;
}
