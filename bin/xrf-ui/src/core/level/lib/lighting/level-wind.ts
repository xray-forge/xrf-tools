import { DEFAULT_RENDERER_TREE_WIND, IRendererTreeWind } from "@xrf/renderer";

/**
 * How the trees of a level preview sway, as the weather's `trees_amplitude` and `trees_speed` would have them.
 */
export interface ILevelWind {
  /** How far a tree leans, a share of its height. */
  windAmplitude: number;
  /** How fast the wave runs through the level. */
  windSpeed: number;
}

/** The engine's own sway, where a weather states none. */
export const DEFAULT_LEVEL_WIND: ILevelWind = {
  windAmplitude: DEFAULT_RENDERER_TREE_WIND.amplitude,
  windSpeed: DEFAULT_RENDERER_TREE_WIND.speed,
};

/** The bounds each value is offered between: still to ten times the engine's lean, still to five times its pace. */
export const LEVEL_WIND_LIMITS = {
  windAmplitude: { max: 0.05, min: 0, step: 0.001 },
  windSpeed: { max: 5, min: 0, step: 0.1 },
} as const;

/**
 * @param wind - The preview's sway, as its controls set it.
 * @returns The renderer's: the engine's turn and wave, at the controls' lean and pace.
 */
export function toLevelRendererTreeWind(wind: ILevelWind): IRendererTreeWind {
  return { ...DEFAULT_RENDERER_TREE_WIND, amplitude: wind.windAmplitude, speed: wind.windSpeed };
}
