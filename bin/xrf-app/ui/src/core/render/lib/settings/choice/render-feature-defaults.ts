import {
  RENDER_DEFAULT_VIEW_FEATURES,
  RENDER_DEFAULT_VIEW_OUTPUT,
  RenderViewFeatures,
  RenderViewOutput,
} from "@xrf/ipc/types/xrf-renderer";

import {
  TRenderAmbientOcclusionGtaoSettings,
  TRenderAmbientOcclusionRtaoSettings,
  TRenderAmbientOcclusionSettings,
  TRenderCameraSettings,
  TRenderContactShadowSettings,
  TRenderDebandingSettings,
  TRenderEnhancedBloomSettings,
  TRenderExposureSettings,
  TRenderFogSettings,
  TRenderFoliageSettings,
  TRenderGlassSettings,
  TRenderGradingSettings,
  TRenderGrassSettings,
  TRenderIndirectLightSettings,
  TRenderLightsSettings,
  TRenderLodSettings,
  TRenderParallaxSettings,
  TRenderRainSettings,
  TRenderReflectionSettings,
  TRenderShadowSettings,
  TRenderTerrainSettings,
  TRenderUpscalingSettings,
  TRenderVolumetricSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { TSettled } from "@/core/render/lib/settings/schema/render-settled";

// Each group as the renderer's contract starts it (its `Default`), which says why each value is what it is.
const FEATURES: TSettled<RenderViewFeatures> = RENDER_DEFAULT_VIEW_FEATURES as TSettled<RenderViewFeatures>;
const OUTPUT: TSettled<RenderViewOutput> = RENDER_DEFAULT_VIEW_OUTPUT as TSettled<RenderViewOutput>;

/** The sun's cascade widths in metres, nearest first. */
export const RENDER_SHADOW_CASCADE_WIDTHS: ReadonlyArray<number> = FEATURES.shadows.cascades;

export const DEFAULT_RENDER_AMBIENT_OCCLUSION_GTAO_SETTINGS: TRenderAmbientOcclusionGtaoSettings =
  FEATURES.ambientOcclusion.gtao;

export const DEFAULT_RENDER_AMBIENT_OCCLUSION_RTAO_SETTINGS: TRenderAmbientOcclusionRtaoSettings =
  FEATURES.ambientOcclusion.rtao;

export const DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS: TRenderAmbientOcclusionSettings = FEATURES.ambientOcclusion;

export const DEFAULT_RENDER_EXPOSURE_SETTINGS: TRenderExposureSettings = FEATURES.exposure;

export const DEFAULT_RENDER_FOG_SETTINGS: TRenderFogSettings = FEATURES.fog;

export const DEFAULT_RENDER_DEBANDING_SETTINGS: TRenderDebandingSettings = FEATURES.debanding;

export const DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS: TRenderEnhancedBloomSettings = FEATURES.enhancedBloom;

export const DEFAULT_RENDER_GRADING_SETTINGS: TRenderGradingSettings = FEATURES.grading;

export const DEFAULT_RENDER_FOLIAGE_SETTINGS: TRenderFoliageSettings = FEATURES.grass.foliage;

export const DEFAULT_RENDER_GLASS_SETTINGS: TRenderGlassSettings = FEATURES.glass;

export const DEFAULT_RENDER_GRASS_SETTINGS: TRenderGrassSettings = FEATURES.grass;

export const DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS: TRenderIndirectLightSettings = FEATURES.indirectLight;

export const DEFAULT_RENDER_RAIN_SETTINGS: TRenderRainSettings = FEATURES.rain;

export const DEFAULT_RENDER_REFLECTION_SETTINGS: TRenderReflectionSettings = FEATURES.reflections;

export const DEFAULT_RENDER_LIGHTS_SETTINGS: TRenderLightsSettings = FEATURES.lights;

export const DEFAULT_RENDER_LOD_SETTINGS: TRenderLodSettings = FEATURES.lod;

export const DEFAULT_RENDER_PARALLAX_SETTINGS: TRenderParallaxSettings = FEATURES.parallax;

export const DEFAULT_RENDER_TERRAIN_SETTINGS: TRenderTerrainSettings = FEATURES.terrain;

export const DEFAULT_RENDER_CAMERA_SETTINGS: TRenderCameraSettings = FEATURES.camera;

export const DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS: TRenderContactShadowSettings = FEATURES.shadows.contact;

export const DEFAULT_RENDER_SHADOW_SETTINGS: TRenderShadowSettings = FEATURES.shadows;

export const DEFAULT_RENDER_UPSCALING_SETTINGS: TRenderUpscalingSettings = OUTPUT.upscaling;

export const DEFAULT_RENDER_VOLUMETRIC_SETTINGS: TRenderVolumetricSettings = FEATURES.volumetric;

export const DEFAULT_RENDER_WATER_SETTINGS: TRenderWaterSettings = FEATURES.water;
