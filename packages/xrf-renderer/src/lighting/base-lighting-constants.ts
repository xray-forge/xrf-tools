import { TRendererColor } from "#/contract/renderer-color";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * What the base lighting passes bind, derived from a lighting value the way the engine derives its constants.
 */
export interface IBaseLightingConstants {
  /** `Ldynamic_color.rgb`: the sun's colour. */
  sunColor: TRendererColor;
  /** `Ldynamic_color.w`: what the sun contributes to specular. */
  sunSpecular: number;
  /** The direction sunlight travels, normalised, in world space. */
  sunDirection: TRendererVector;
  /** `L_ambient`. */
  ambient: TRendererColor;
  /** `env_color.rgb` as combine binds it. */
  environment: TRendererColor;
  /** What the irradiance cube returns. */
  skyIrradiance: TRendererColor;
  /** `fog_params.x` and `.w`: fog is `saturate(distance * w + x)`. Zero and zero is no fog. */
  fogOffset: number;
  fogScale: number;
  fogColor: TRendererColor;
  /** Whether there is fog, and so a distance past which it hides everything. */
  isFogged: boolean;
}
