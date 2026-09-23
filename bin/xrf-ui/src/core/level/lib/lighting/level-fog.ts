import { IRendererFog, TRendererColor } from "@xrf/renderer";

/**
 * How a level preview is fogged, which its controls set the way they set the sun.
 */
export interface ILevelFog {
  /** Metres out where fog is total, and where the view ends. */
  fogDistance: number;
  /** How near the eye fog starts: zero at 85% of its distance, one at the eye. */
  fogDensity: number;
  /** What the noon fog's colour is scaled by. */
  fogIntensity: number;
}

/** `fog_color` of `default_clear` at noon (`configs/environment/weathers/default_clear.ltx`, `[12:00:00]`). */
const LEVEL_NOON_FOG_COLOR: TRendererColor = [0.304609, 0.328138, 0.367354];

/** `default_clear`'s noon fog: total at 350 metres, starting at a tenth of 85% of that. */
export const DEFAULT_LEVEL_FOG: ILevelFog = {
  fogDensity: 0.9,
  fogDistance: 350,
  fogIntensity: 1,
};

/** The bounds each value is offered between. */
export const LEVEL_FOG_LIMITS = {
  fogDensity: { max: 1, min: 0, step: 0.05 },
  fogDistance: { max: 2000, min: 50, step: 10 },
  fogIntensity: { max: 4, min: 0, step: 0.05 },
} as const;

/**
 * @param fog - The fog as its controls set it.
 * @returns The fog the renderer draws: the noon colour, scaled, at the distance and density asked for.
 */
export function toLevelRendererFog(fog: ILevelFog): IRendererFog {
  return {
    color: [
      LEVEL_NOON_FOG_COLOR[0] * fog.fogIntensity,
      LEVEL_NOON_FOG_COLOR[1] * fog.fogIntensity,
      LEVEL_NOON_FOG_COLOR[2] * fog.fogIntensity,
    ],
    density: fog.fogDensity,
    distance: fog.fogDistance,
  };
}
