import { TRendererColor } from "#/contract/renderer-color";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * One `level.env_mod` volume (`CEnvModifier`): where it is, how far and how strongly it reaches, and what it adds of
 * each value its flags name.
 */
export interface IRendererWeatherModifier {
  /** In engine space. */
  position: TRendererVector;
  radius: number;
  power: number;
  farPlane: number;
  fogColor: TRendererColor;
  fogDensity: number;
  ambient: TRendererColor;
  skyColor: TRendererColor;
  hemiColor: TRendererColor;
  /** `EEnvModUsedParams`: which values it adds to, every one for a file older than the flags. */
  flags: number;
}
