import {
  RenderAmbientOcclusionSettings,
  RenderExposureSettings,
  RenderGrassSettings,
  RenderLightsSettings,
  RenderLodSettings,
  RenderShadowSettings,
  RenderUpscalingSettings,
  RenderViewOptions,
  RenderWaterSettings,
} from "@/core/ipc/types/xrf-renderer";
import { TSettled } from "@/core/render/lib/settings/render-settled";

/** The view options every viewport takes from the application's settings rather than from its own toolbar. */
export type TRenderFeatureKey =
  | "ambientOcclusion"
  | "antialiasing"
  | "exposure"
  | "grass"
  | "isOcclusionCulled"
  | "lights"
  | "lod"
  | "shadows"
  | "upscaling"
  | "water";

/**
 * What the renderer's features are set to: the same for every viewport, chosen as a preset and whatever was changed
 * on top of it. A feature that is off costs nothing: its passes leave the frame and its targets are freed.
 */
export interface IRenderFeatureSettings extends TSettled<Pick<RenderViewOptions, TRenderFeatureKey>> {}

export type TRenderAmbientOcclusionSettings = TSettled<RenderAmbientOcclusionSettings>;

export type TRenderExposureSettings = TSettled<RenderExposureSettings>;

export type TRenderGrassSettings = TSettled<RenderGrassSettings>;

export type TRenderLightsSettings = TSettled<RenderLightsSettings>;

export type TRenderLodSettings = TSettled<RenderLodSettings>;

export type TRenderShadowSettings = TSettled<RenderShadowSettings>;

export type TRenderUpscalingSettings = TSettled<RenderUpscalingSettings>;

export type TRenderWaterSettings = TSettled<RenderWaterSettings>;
