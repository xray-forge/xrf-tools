import { IRendererAmbientOcclusionSettings } from "#/contract/renderer-ambient-occlusion-settings";
import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { IRendererExposureSettings } from "#/contract/renderer-exposure-settings";
import { IRendererGrassSettings } from "#/contract/renderer-grass-settings";
import { IRendererLightsSettings } from "#/contract/renderer-lights-settings";
import { IRendererLodSettings } from "#/contract/renderer-lod-settings";
import { RENDERER_RENDER_SCALE_RATIOS } from "#/contract/renderer-render-scale";
import { IRendererShadowSettings } from "#/contract/renderer-shadow-settings";
import { IRendererUpscalingSettings } from "#/contract/renderer-upscaling-settings";
import { IRendererWaterSettings } from "#/contract/renderer-water-settings";

/**
 * What the renderer's features are set to: the same for every consumer, chosen as a preset and whatever was changed
 * on top of it. A feature that is off costs nothing: its passes leave the frame and its targets are freed.
 */
export interface IRendererFeatureSettings {
  ambientOcclusion: IRendererAmbientOcclusionSettings;
  antialiasing: ERendererAntialiasing;
  exposure: IRendererExposureSettings;
  grass: IRendererGrassSettings;
  /** Whether every pass is timed on the GPU for the report. */
  isGpuTimed: boolean;
  /**
   * Whether the static draws the depth hides are culled, in two phases: against the last frame's depth, then what that
   * hid against this frame's so far. Off, every static draw the frustum keeps is drawn, at no pyramid and no second
   * phase.
   */
  isOcclusionCulled: boolean;
  lights: IRendererLightsSettings;
  lod: IRendererLodSettings;
  shadows: IRendererShadowSettings;
  upscaling: IRendererUpscalingSettings;
  water: IRendererWaterSettings;
}

/**
 * @param features - What the features are set to.
 * @returns The ratio of the output's side to the scene's as drawn.
 */
export function toRendererUpscale(features: IRendererFeatureSettings): number {
  return RENDERER_RENDER_SCALE_RATIOS[features.upscaling.scale];
}
