import { ERendererRenderScale } from "#/contract/renderer-render-scale";

/**
 * What the scene is drawn at: a share of the output, upscaled, and the output sharpened after while it is upscaled.
 */
export interface IRendererUpscalingSettings {
  scale: ERendererRenderScale;
  /** RCAS's sharpness, from none to its most: zero leaves the upscaled frame as resolved. */
  sharpening: number;
}

/** Drawn at the output's size; sharpened halfway once upscaled. */
export const DEFAULT_RENDERER_UPSCALING_SETTINGS: IRendererUpscalingSettings = {
  scale: ERendererRenderScale.NATIVE,
  sharpening: 0.5,
};
