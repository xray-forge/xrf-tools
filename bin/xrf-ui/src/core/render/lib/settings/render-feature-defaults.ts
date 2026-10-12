import {
  ERenderAmbientOcclusionMethod,
  ERenderAmbientOcclusionQuality,
  ERenderBloomMode,
  ERenderContactShadowMode,
  ERenderDebandingMode,
  ERenderDebandingQuality,
  ERenderFogMode,
  ERenderFoliageMode,
  ERenderGlassMode,
  ERenderIndirectLightMode,
  ERenderLightShadowFilter,
  ERenderParallaxMode,
  ERenderRainMode,
  ERenderReflectionMode,
  ERenderReflectionProbe,
  ERenderReflectionQuality,
  ERenderReflectionTraceQuality,
  ERenderScale,
  ERenderVolumetricMode,
  ERenderWaterMode,
} from "@/core/ipc/types/xrf-renderer";
import {
  TRenderAmbientOcclusionSettings,
  TRenderAmbientOcclusionVbaoSettings,
  TRenderContactShadowSettings,
  TRenderDebandingSettings,
  TRenderEnhancedBloomSettings,
  TRenderEnhancedWaterSettings,
  TRenderExposureSettings,
  TRenderFogSettings,
  TRenderFoliageSettings,
  TRenderGlassSettings,
  TRenderGrassSettings,
  TRenderIndirectLightSettings,
  TRenderLightsSettings,
  TRenderLodSettings,
  TRenderParallaxSettings,
  TRenderRainSettings,
  TRenderReflectionSettings,
  TRenderShadowSettings,
  TRenderUpscalingSettings,
  TRenderVolumetricSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/render-feature-settings";

/** The engine's three cascade widths in metres (`render_phase_sun.cpp`), and a fourth reaching three times as far. */
export const RENDER_SHADOW_CASCADE_WIDTHS: ReadonlyArray<number> = [20, 40, 160, 480];

/** Cascades the sun's shadow can be cut into at most: the sun reads their texels from one `vec4`. */
export const RENDER_MAX_SHADOW_CASCADES: number = 4;

/** Local lights a pixel marches contact shadows towards at most: the lights' shader keeps their weights in an array. */
export const RENDER_MAX_CONTACT_SHADOW_LIGHTS: number = 8;

/** Frames VBAO accumulates over at most. */
export const RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION: number = 32;

/** VBAO's strengths: a grass clump's depth, every bounce, and eight frames. */
export const DEFAULT_RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS: TRenderAmbientOcclusionVbaoSettings = {
  accumulation: 8,
  bounce: 1,
  thickness: 0.25,
};

/** XeGTAO's defaults, at a metre, and VBAO's strengths for when it searches. */
export const DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS: TRenderAmbientOcclusionSettings = {
  isEnabled: true,
  method: ERenderAmbientOcclusionMethod.GTAO,
  quality: ERenderAmbientOcclusionQuality.HIGH,
  radius: 1,
  strength: 1,
  vbao: DEFAULT_RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS,
};

/** OpenXRay's own exposure (`xrRender_console.cpp`). */
export const DEFAULT_RENDER_EXPOSURE_SETTINGS: TRenderExposureSettings = {
  adaptation: 1,
  amount: 0.7,
  isEnabled: true,
  lowLuminance: 0.0001,
  middleGray: 1,
};

/** The engine's own fog, with the enhanced fog's shipped strengths but no scattering for when it is drawn. */
export const DEFAULT_RENDER_FOG_SETTINGS: TRenderFogSettings = {
  density: 1.3,
  height: 8,
  mode: ERenderFogMode.ENGINE,
  scattering: 0,
  sunColor: 0.1,
};

/** The sky as drawn, with the debanding's shipped strengths for when it is debanded: two passes out to 48 pixels. */
export const DEFAULT_RENDER_DEBANDING_SETTINGS: TRenderDebandingSettings = {
  mode: ERenderDebandingMode.ENGINE,
  quality: ERenderDebandingQuality.MEDIUM,
  radius: 48,
};

/** The engine's own bloom, as the look sets it, with the enhanced bloom's shipped strengths for when it draws. */
export const DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS: TRenderEnhancedBloomSettings = {
  blur: 3,
  exposure: 3,
  mode: ERenderBloomMode.ENGINE,
  sky: 0.6,
  threshold: 5,
  vibrance: 1,
};

/** The engine's own foliage motion, with the enhanced motion's designed strengths. */
export const DEFAULT_RENDER_FOLIAGE_SETTINGS: TRenderFoliageSettings = {
  grassPush: 1.5,
  grassSpeed: 9.5,
  grassTurbulence: 1.4,
  grassWave: 0.4,
  minSpeed: 0.1,
  mode: ERenderFoliageMode.ENGINE,
  sssColor: 0.75,
  sssIntensity: 1,
  treesBend: 0.5,
  treesSpeed: 11,
  treesTrunk: 0.15,
};

/** The engine's own glass, with the enhanced glass's designed refraction for when it is drawn. */
export const DEFAULT_RENDER_GLASS_SETTINGS: TRenderGlassSettings = {
  mode: ERenderGlassMode.ENGINE,
  refraction: 1,
};

/** The engine's own grass. */
export const DEFAULT_RENDER_GRASS_SETTINGS: TRenderGrassSettings = {
  density: 0.6,
  foliage: DEFAULT_RENDER_FOLIAGE_SETTINGS,
  height: 1,
  isAligned: false,
  isEnabled: true,
  radius: 49,
  slopeLimit: 90,
};

/** The engine's own: no bounced light, and all of it from three metres around for when it is gathered. */
export const DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS: TRenderIndirectLightSettings = {
  intensity: 1,
  mode: ERenderIndirectLightMode.ENGINE,
  radius: 3,
};

/** The engine's own wetting; enhanced, half the puddle sites filled to 300 m, each taking up to 40% reflection. */
export const DEFAULT_RENDER_RAIN_SETTINGS: TRenderRainSettings = {
  distance: 300,
  mode: ERenderRainMode.ENGINE,
  puddles: 1,
  reflectivity: 0.4,
  ripples: 1,
};

/**
 * The engine's own: the cube alone, and for when it is traced, rays of 150 metres at high quality at full strength over
 * ground wet to 300 metres, the sky's highlights a quarter brighter as their light.
 */
export const DEFAULT_RENDER_REFLECTION_SETTINGS: TRenderReflectionSettings = {
  distance: 150,
  reach: 300,
  intensity: 1,
  mode: ERenderReflectionMode.ENGINE,
  probe: ERenderReflectionProbe.FINE,
  quality: ERenderReflectionQuality.HIGH,
  skyGain: 1.25,
  isGrassReflected: false,
  isTraced: false,
  traceQuality: ERenderReflectionTraceQuality.HIGH,
};

/** The engine's own: every spawned light, and none of the level file's; no shafts until they cost less. */
export const DEFAULT_RENDER_LIGHTS_SETTINGS: TRenderLightsSettings = {
  isEnabled: true,
  isLevelLights: false,
  isShadowed: true,
  shadowFilter: ERenderLightShadowFilter.ENGINE,
  isVolumetric: true,
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

/**
 * The engines' own, steep as Anomaly's high and extreme presets set `r2_steep_parallax`, with the enhanced parallax's
 * strengths for when it is drawn: sixteen steps within 60 m, 0.02 of the detail deep, refined, self-shadowed, its layers
 * half blended by height.
 */
export const DEFAULT_RENDER_PARALLAX_SETTINGS: TRenderParallaxSettings = {
  blending: 0.5,
  cavity: 1,
  height: 0.02,
  isRefined: true,
  isSteep: true,
  mode: ERenderParallaxMode.ENGINE,
  range: 60,
  shadow: 1,
  steps: 16,
  surfaceHeight: 0.02,
  surfaceRange: 60,
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

/** The engine's own shafts, with the froxel volume at half its designed strength and a light lean, out to 120 m. */
export const DEFAULT_RENDER_VOLUMETRIC_SETTINGS: TRenderVolumetricSettings = {
  anisotropy: 0.15,
  distance: 120,
  intensity: 0.5,
  mode: ERenderVolumetricMode.ENGINE,
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
  vibrance: 1,
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
