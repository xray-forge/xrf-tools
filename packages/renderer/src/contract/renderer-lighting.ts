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
}
