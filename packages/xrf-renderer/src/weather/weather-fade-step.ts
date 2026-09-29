import { IRendererLighting } from "#/contract/renderer-lighting";

/**
 * One step of a fade from what was shown to what the weather shows now.
 */
export interface IWeatherFadeStep {
  /** What was shown as the fade started, which stays as it was. */
  from: IRendererLighting;
  /** What the weather shows now, which may move while the fade runs. */
  to: IRendererLighting;
  /** How far from the one to the other, from nought to one. */
  progress: number;
}
