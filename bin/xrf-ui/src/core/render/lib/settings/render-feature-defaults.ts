import {
  ERenderAmbientOcclusionQuality,
  ERenderContactShadowMode,
  ERenderLightShadowFilter,
  ERenderScale,
  ERenderWaterMode,
} from "@/core/ipc/types/xrf-renderer";
import {
  TRenderAmbientOcclusionSettings,
  TRenderContactShadowSettings,
  TRenderEnhancedWaterSettings,
  TRenderExposureSettings,
  TRenderGrassSettings,
  TRenderLightsSettings,
  TRenderLodSettings,
  TRenderShadowSettings,
  TRenderUpscalingSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/render-feature-settings";

/** The engine's three cascade widths in metres (`render_phase_sun.cpp`), and a fourth reaching three times as far. */
export const RENDER_SHADOW_CASCADE_WIDTHS: ReadonlyArray<number> = [20, 40, 160, 480];

/** Cascades the sun's shadow can be cut into at most: the sun reads their texels from one `vec4`. */
export const RENDER_MAX_SHADOW_CASCADES: number = 4;

/** Local lights a pixel marches contact shadows towards at most: the lights' shader keeps their weights in an array. */
export const RENDER_MAX_CONTACT_SHADOW_LIGHTS: number = 8;

/** XeGTAO's defaults, at a metre. */
export const DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS: TRenderAmbientOcclusionSettings = {
  isEnabled: true,
  quality: ERenderAmbientOcclusionQuality.HIGH,
  radius: 1,
  strength: 1,
};

/** OpenXRay's own exposure (`xrRender_console.cpp`). */
export const DEFAULT_RENDER_EXPOSURE_SETTINGS: TRenderExposureSettings = {
  adaptation: 1,
  amount: 0.7,
  isEnabled: true,
  lowLuminance: 0.0001,
  middleGray: 1,
};

/** The engine's own grass. */
export const DEFAULT_RENDER_GRASS_SETTINGS: TRenderGrassSettings = {
  density: 0.6,
  height: 1,
  isEnabled: true,
  radius: 49,
};

/** The engine's own: every spawned light, and none of the level file's. */
export const DEFAULT_RENDER_LIGHTS_SETTINGS: TRenderLightsSettings = {
  isEnabled: true,
  isLevelLights: false,
  isShadowed: true,
  shadowFilter: ERenderLightShadowFilter.ENGINE,
};

/** The engine's own values (`xrRender_console.cpp`). */
export const DEFAULT_RENDER_LOD_SETTINGS: TRenderLodSettings = {
  geometryLod: 0.75,
  isImpostors: true,
  ssaA: 64,
  ssaB: 48,
  ssaDiscard: 3.5,
  ssaGlodEnd: 64,
  ssaGlodStart: 256,
};

/** The engine's own: no contact shadows, and their strengths for when they are drawn, towards four lights a pixel. */
export const DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS: TRenderContactShadowSettings = {
  intensity: 1,
  length: 0.6,
  lights: 4,
  mode: ERenderContactShadowMode.ENGINE,
  steps: 16,
  thickness: 0.1,
};

/** Every cascade, the engine's three and the far one, a filter a texel wide, and no contact shadows. */
export const DEFAULT_RENDER_SHADOW_SETTINGS: TRenderShadowSettings = {
  bias: 1.5,
  blend: 0.1,
  cascades: RENDER_SHADOW_CASCADE_WIDTHS,
  contact: DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS,
  filter: 1,
  isEnabled: true,
  isStaggered: true,
  reach: 400,
  resolution: 2048,
};

/** Drawn at the viewport's size; sharpened halfway once upscaled. */
export const DEFAULT_RENDER_UPSCALING_SETTINGS: TRenderUpscalingSettings = {
  scale: ERenderScale.NATIVE,
  sharpening: 0.5,
};

/**
 * The enhanced water's designed strengths, but for caustics a sixth as bright, calm water flowing a fifth as fast, and
 * its maps' repeat broken.
 */
export const DEFAULT_RENDER_ENHANCED_WATER_SETTINGS: TRenderEnhancedWaterSettings = {
  blurNoise: 1,
  calmFlow: 0.2,
  caustics: 0.05,
  flow: 1,
  parallaxHeight: 0.05,
  reflectionBlur: 0.8,
  reflectivity: 0.8,
  refraction: 0.6,
  ripples: 0.5,
  softBorder: 0.05,
  specular: 6,
  turbidity: 3,
  variation: 0.75,
};

/** The engine's own water: `shared/waterconfig.h`'s constants and `def_distort`. */
export const DEFAULT_RENDER_WATER_SETTINGS: TRenderWaterSettings = {
  distortion: 0.05,
  enhanced: DEFAULT_RENDER_ENHANCED_WATER_SETTINGS,
  isDistorted: true,
  isEnabled: true,
  isSoft: true,
  mode: ERenderWaterMode.ENGINE,
  reflection: 1,
  ripple: 1,
  waveHeight: 1 / 60,
  waveSpeed: 25,
};
