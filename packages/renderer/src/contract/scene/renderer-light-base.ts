import { TRendererColor } from "#/contract/renderer-color";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * What every local light is, as the engine hands it to its shaders: accumulated after the sun, `Ldynamic_color` times
 * `plight_local`'s falloff to its range.
 */
export interface IRendererLightBase {
  position: TRendererVector;
  /** Raw, as the engine sets it: a lamp's colour times its brightness. */
  color: TRendererColor;
  range: number;
  /** How far the range strays each frame, either way, at random: a zone's flicker. None where left out. */
  rangeJitter?: number;
  /** Where its shadow faces' projection starts; the engine's default where zero. */
  near: number;
  /** The animation replacing its colour, by its index among the lights' animators. */
  animator?: number;
  /** What an animated colour, each channel in `[0, 255]`, is multiplied by. */
  animatorScale: number;
  /** Whether it casts shadows, which also has it fade and drop out with distance as the engine's shadowed lights do. */
  isShadowed: boolean;
  /** Whether it is one of the level file's own lights, which the engine draws only with `r2_allow_r1_lights`. */
  isLevel: boolean;
}
