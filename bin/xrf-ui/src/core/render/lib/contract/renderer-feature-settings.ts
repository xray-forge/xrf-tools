import { IRendererAmbientOcclusionSettings } from "@/core/render/lib/contract/renderer-ambient-occlusion-settings";
import { ERendererAntialiasing } from "@/core/render/lib/contract/renderer-antialiasing";
import { IRendererExposureSettings } from "@/core/render/lib/contract/renderer-exposure-settings";
import { IRendererGrassSettings } from "@/core/render/lib/contract/renderer-grass-settings";
import { IRendererLightsSettings } from "@/core/render/lib/contract/renderer-lights-settings";
import { IRendererLodSettings } from "@/core/render/lib/contract/renderer-lod-settings";
import { RENDERER_RENDER_SCALE_RATIOS } from "@/core/render/lib/contract/renderer-render-scale";
import { IRendererShadowSettings } from "@/core/render/lib/contract/renderer-shadow-settings";
import { IRendererUpscalingSettings } from "@/core/render/lib/contract/renderer-upscaling-settings";
import { IRendererWaterSettings } from "@/core/render/lib/contract/renderer-water-settings";

/**
 * What the renderer's features are set to: the same for every consumer, chosen as a preset and whatever was changed
 * on top of it. A feature that is off costs nothing: its passes leave the frame and its targets are freed.
 */
export interface IRendererFeatureSettings {
  ambientOcclusion: IRendererAmbientOcclusionSettings;
  antialiasing: ERendererAntialiasing;
  exposure: IRendererExposureSettings;
  grass: IRendererGrassSettings;
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
