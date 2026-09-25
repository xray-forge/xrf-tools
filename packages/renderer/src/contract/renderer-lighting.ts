import { Nullable } from "@xrf/types";

/** Red, green and blue, each as an engine config states it. */
export type TRendererColor = readonly [number, number, number];

/** A direction or a point in renderer space. */
export type TRendererVector = readonly [number, number, number];

/**
 * Distance fog, as a weather keyframe states it.
 */
export interface IRendererFog {
  /** `fog_color`. */
  color: TRendererColor;
  /** `fog_distance`, in metres: where fog is total. */
  distance: number;
  /** `fog_density`, from zero (fog starts at 85% of the distance) to one (it starts at the eye). */
  density: number;
}

/**
 * How the trees sway, in the terms a weather keyframe uses (`CEnvDescriptor::m_fTree*`).
 */
export interface IRendererTreeWind {
  /** `trees_amplitude`: how far a tree leans, a share of its height. */
  amplitude: number;
  /** `trees_speed`: how fast the wave runs through the level. */
  speed: number;
  /** `trees_rotation`: seconds the wind takes to turn once around. */
  rotation: number;
  /** `trees_wave`: the wave's direction through the level, in the engine's own axes. */
  wave: TRendererVector;
}

/**
 * One way the grass swings, as `[details]` in `system.ltx` states it: two winds turning at their own rates, and a wave
 * running through the level.
 */
export interface IRendererGrassSwing {
  /** How far the first and second wind lean the grass. */
  amp1: number;
  amp2: number;
  /** Seconds each wind takes to turn once around. */
  rot1: number;
  rot2: number;
  /** How fast the wave runs. */
  speed: number;
}

/**
 * How the grass sways (`CDetailManager::swing_desc`): between its normal and its fast swing, by the weather's wind
 * strength.
 */
export interface IRendererGrassWind {
  /** `wind_strength_factor`, from the normal swing at zero to the fast one at one. */
  strength: number;
  normal: IRendererGrassSwing;
  fast: IRendererGrassSwing;
}

/**
 * What a scene is lit by, in the terms a weather keyframe uses.
 */
export interface IRendererLighting {
  /** The direction sunlight travels, in renderer space. */
  sunDirection: TRendererVector;
  /** `sun_color`. */
  sunColor: TRendererColor;
  /** `hemisphere_color`. */
  hemisphereColor: TRendererColor;
  /** `ambient_color`. */
  ambientColor: TRendererColor;
  /** What the sky's irradiance cube returns, standing in for one until a weather supplies it. */
  skyIrradiance: TRendererColor;
  /** Distance fog, or none. */
  fog: Nullable<IRendererFog>;
  /** How the trees sway, or null for trees standing still. */
  trees: Nullable<IRendererTreeWind>;
  /** How the grass sways, or null for grass standing still. */
  grass: Nullable<IRendererGrassWind>;
}
