import { TRendererVector } from "#/contract/renderer-vector";

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
