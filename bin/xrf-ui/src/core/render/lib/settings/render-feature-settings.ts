import {
  RenderAmbientOcclusionGtaoSettings,
  RenderAmbientOcclusionRtaoSettings,
  RenderAmbientOcclusionSettings,
  RenderAmbientOcclusionVbaoSettings,
  RenderContactShadowSettings,
  RenderDebandingSettings,
  RenderEnhancedBloomSettings,
  RenderEnhancedWaterSettings,
  RenderExposureSettings,
  RenderFogSettings,
  RenderFoliageSettings,
  RenderGlassSettings,
  RenderGradingSettings,
  RenderGrassSettings,
  RenderIndirectLightSettings,
  RenderLightsSettings,
  RenderLodSettings,
  RenderParallaxSettings,
  RenderRainSettings,
  RenderReflectionSettings,
  RenderSettings,
  RenderShadowSettings,
  RenderUpscalingSettings,
  RenderViewFeatures,
  RenderViewOutput,
  RenderVolumetricSettings,
  RenderWaterSettings,
} from "@/core/ipc/types/xrf-renderer";
import { TSettled } from "@/core/render/lib/settings/render-settled";

/** The features a viewport resolves from the application's settings, a view overriding some of them. */
export type TRenderViewFeatureKey =
  | "ambientOcclusion"
  | "antialiasing"
  | "debanding"
  | "displayTransform"
  | "enhancedBloom"
  | "exposure"
  | "fog"
  | "glass"
  | "grading"
  | "grass"
  | "indirectLight"
  | "isDepthPrepassed"
  | "isOcclusionCulled"
  | "lightLevel"
  | "lightModel"
  | "lights"
  | "lod"
  | "parallax"
  | "rain"
  | "reflections"
  | "shadows"
  | "sunTint"
  | "tonemap"
  | "volumetric"
  | "water";

/**
 * What the renderer's features are set to: the same for every viewport, chosen as a preset and whatever was changed
 * on top of it. A feature that is off costs nothing: its passes leave the frame and its targets are freed.
 */
export interface IRenderFeatureSettings extends TSettled<
  Pick<RenderViewFeatures, TRenderViewFeatureKey> &
    Pick<RenderViewOutput, "upscaling"> &
    Pick<RenderSettings, "textureFiltering" | "textureLod">
> {}

export type TRenderAmbientOcclusionVbaoSettings = TSettled<RenderAmbientOcclusionVbaoSettings>;

export type TRenderAmbientOcclusionGtaoSettings = TSettled<RenderAmbientOcclusionGtaoSettings>;

export type TRenderAmbientOcclusionRtaoSettings = TSettled<RenderAmbientOcclusionRtaoSettings>;

export type TRenderAmbientOcclusionSettings = TSettled<RenderAmbientOcclusionSettings>;

export type TRenderContactShadowSettings = TSettled<RenderContactShadowSettings>;

export type TRenderDebandingSettings = TSettled<RenderDebandingSettings>;

export type TRenderEnhancedBloomSettings = TSettled<RenderEnhancedBloomSettings>;

export type TRenderGradingSettings = TSettled<RenderGradingSettings>;

export type TRenderEnhancedWaterSettings = TSettled<RenderEnhancedWaterSettings>;

export type TRenderExposureSettings = TSettled<RenderExposureSettings>;

export type TRenderFogSettings = TSettled<RenderFogSettings>;

export type TRenderFoliageSettings = TSettled<RenderFoliageSettings>;

export type TRenderGlassSettings = TSettled<RenderGlassSettings>;

export type TRenderGrassSettings = TSettled<RenderGrassSettings>;

export type TRenderIndirectLightSettings = TSettled<RenderIndirectLightSettings>;

export type TRenderLightsSettings = TSettled<RenderLightsSettings>;

export type TRenderLodSettings = TSettled<RenderLodSettings>;

export type TRenderParallaxSettings = TSettled<RenderParallaxSettings>;

export type TRenderRainSettings = TSettled<RenderRainSettings>;

export type TRenderReflectionSettings = TSettled<RenderReflectionSettings>;

export type TRenderShadowSettings = TSettled<RenderShadowSettings>;

export type TRenderUpscalingSettings = TSettled<RenderUpscalingSettings>;

export type TRenderVolumetricSettings = TSettled<RenderVolumetricSettings>;

export type TRenderWaterSettings = TSettled<RenderWaterSettings>;
