import {
  RenderAmbientOcclusionSettings,
  RenderAmbientOcclusionVbaoSettings,
  RenderContactShadowSettings,
  RenderEnhancedWaterSettings,
  RenderExposureSettings,
  RenderFoliageSettings,
  RenderGrassSettings,
  RenderIndirectLightSettings,
  RenderLightsSettings,
  RenderLodSettings,
  RenderShadowSettings,
  RenderUpscalingSettings,
  RenderViewFeatures,
  RenderViewOutput,
  RenderWaterSettings,
} from "@/core/ipc/types/xrf-renderer";
import { TSettled } from "@/core/render/lib/settings/render-settled";

/** The features every viewport takes from the application's settings rather than from its own toolbar. */
export type TRenderViewFeatureKey =
  | "ambientOcclusion"
  | "antialiasing"
  | "exposure"
  | "grass"
  | "indirectLight"
  | "isOcclusionCulled"
  | "lights"
  | "lod"
  | "shadows"
  | "water";

/** The view options every viewport takes from the application's settings: those features, and how it upscales. */
export type TRenderFeatureKey = TRenderViewFeatureKey | "upscaling";

/**
 * What the renderer's features are set to: the same for every viewport, chosen as a preset and whatever was changed
 * on top of it. A feature that is off costs nothing: its passes leave the frame and its targets are freed.
 */
export interface IRenderFeatureSettings extends TSettled<
  Pick<RenderViewFeatures, TRenderViewFeatureKey> & Pick<RenderViewOutput, "upscaling">
> {}

export type TRenderAmbientOcclusionVbaoSettings = TSettled<RenderAmbientOcclusionVbaoSettings>;

export type TRenderAmbientOcclusionSettings = TSettled<RenderAmbientOcclusionSettings>;

export type TRenderContactShadowSettings = TSettled<RenderContactShadowSettings>;

export type TRenderEnhancedWaterSettings = TSettled<RenderEnhancedWaterSettings>;

export type TRenderExposureSettings = TSettled<RenderExposureSettings>;

export type TRenderFoliageSettings = TSettled<RenderFoliageSettings>;

export type TRenderGrassSettings = TSettled<RenderGrassSettings>;

export type TRenderIndirectLightSettings = TSettled<RenderIndirectLightSettings>;

export type TRenderLightsSettings = TSettled<RenderLightsSettings>;

export type TRenderLodSettings = TSettled<RenderLodSettings>;

export type TRenderShadowSettings = TSettled<RenderShadowSettings>;

export type TRenderUpscalingSettings = TSettled<RenderUpscalingSettings>;

export type TRenderWaterSettings = TSettled<RenderWaterSettings>;
