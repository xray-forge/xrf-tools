import { Nullable } from "@xrf/types";

import { ERendererAmbientOcclusionQuality } from "#/contract/renderer-ambient-occlusion-quality";
import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { IRendererFeatureSettings, toRendererUpscale } from "#/contract/renderer-feature-settings";
import { toShadowCascadeCount } from "#/contract/renderer-shadow-settings";
import { isRendererSmoothing, TRendererSmoothingAntialiasing } from "#/contract/renderer-smoothing-antialiasing";
import { isRendererTemporal, TRendererTemporalAntialiasing } from "#/contract/renderer-temporal-antialiasing";
import { IFramePlanShadows } from "#/graph/frame-plan-shadows";

/**
 * Which of the optional stages a frame draws, and how: what the features come to, before any pass is made.
 */
export interface IFramePlan {
  /** The output's side over the scene's as drawn. */
  readonly upscale: number;
  /** The temporal resolve, which jitters the scene, upscales it and has the sky's motion written. */
  readonly resolve: Nullable<TRendererTemporalAntialiasing>;
  readonly smoothing: Nullable<TRendererSmoothingAntialiasing>;
  /** Whether FSR 1 upscales what the smoothing finished: an upscaled frame with no resolve. */
  readonly isSpatial: boolean;
  /** RCAS over the upscaled frame, with FSR 2's denoise under FSR 2. */
  readonly sharpen: Nullable<{ readonly isDenoised: boolean }>;
  readonly isGrassy: boolean;
  /** Whether the exposure adapts to the frame. */
  readonly isExposed: boolean;
  readonly isLit: boolean;
  readonly isLightShadowed: boolean;
  /** Whether the static draws the depth hides are culled: the second cull, its draw and the pyramid. */
  readonly isOccluding: boolean;
  /** Whether the water draws, its targets allocated. */
  readonly isWatered: boolean;
  /** Whether the water moves what is seen through it, a copy of the frame allocated. */
  readonly isDistorted: boolean;
  readonly ambientOcclusion: Nullable<ERendererAmbientOcclusionQuality>;
  readonly shadows: Nullable<IFramePlanShadows>;
}

/**
 * @param features - What the features are set to.
 * @returns The frame they make.
 */
export function toFramePlan(features: IRendererFeatureSettings): IFramePlan {
  const { antialiasing, shadows, lights } = features;
  const upscale: number = toRendererUpscale(features);
  const resolve: Nullable<TRendererTemporalAntialiasing> = isRendererTemporal(antialiasing) ? antialiasing : null;
  const smoothing: Nullable<TRendererSmoothingAntialiasing> = isRendererSmoothing(antialiasing) ? antialiasing : null;
  const count: number = toShadowCascadeCount(shadows);

  return {
    ambientOcclusion: features.ambientOcclusion.isEnabled ? features.ambientOcclusion.quality : null,
    isDistorted: features.water.isEnabled && features.water.isDistorted,
    isExposed: features.exposure.isEnabled,
    isGrassy: features.grass.isEnabled,
    isLightShadowed: lights.isEnabled && lights.isShadowed,
    isLit: lights.isEnabled,
    isOccluding: features.isOcclusionCulled,
    isSpatial: resolve === null && upscale > 1,
    isWatered: features.water.isEnabled,
    resolve,
    shadows: count > 0 ? { count, resolution: shadows.resolution } : null,
    sharpen:
      upscale > 1 && features.upscaling.sharpening > 0
        ? { isDenoised: antialiasing === ERendererAntialiasing.FSR2 }
        : null,
    smoothing,
    upscale,
  };
}
