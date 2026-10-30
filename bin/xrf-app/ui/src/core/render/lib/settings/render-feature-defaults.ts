import {
  ERenderAmbientOcclusionMethod,
  ERenderBloomMode,
  ERenderContactShadowMode,
  ERenderDebandingMode,
  ERenderDlssPreset,
  ERenderFogColour,
  ERenderFogMode,
  ERenderFoliageMode,
  ERenderGlassMode,
  ERenderIndirectLightMode,
  ERenderLightFalloff,
  ERenderLightShadowFilter,
  ERenderParallaxMode,
  ERenderRainDropsMode,
  ERenderRainMode,
  ERenderRayReconstructionPreset,
  ERenderReflectionMode,
  ERenderReflectionProbe,
  ERenderScale,
  ERenderTerrainTiling,
  ERenderTraceSize,
  ERenderVolumetricMode,
  ERenderWaterMode,
} from "@xrf/ipc/types/xrf-renderer";

import {
  TRenderAmbientOcclusionGtaoSettings,
  TRenderAmbientOcclusionRtaoSettings,
  TRenderAmbientOcclusionSettings,
  TRenderAmbientOcclusionVbaoSettings,
  TRenderCameraSettings,
  TRenderContactShadowSettings,
  TRenderDebandingSettings,
  TRenderEnhancedBloomSettings,
  TRenderEnhancedWaterSettings,
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
} from "@/core/render/lib/settings/render-feature-settings";

/** The engine's three cascade widths in metres (`render_phase_sun.cpp`), and a fourth reaching three times as far. */
export const RENDER_SHADOW_CASCADE_WIDTHS: ReadonlyArray<number> = [20, 40, 160, 480];

/** VBAO's strengths: a grass clump's depth, every bounce, and eight frames. */
const DEFAULT_RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS: TRenderAmbientOcclusionVbaoSettings = {
  accumulation: 8,
  bounce: 1,
  slices: 1,
  steps: 8,
  thickness: 0.25,
};

/** XeGTAO's High: three slices of three steps. */
export const DEFAULT_RENDER_AMBIENT_OCCLUSION_GTAO_SETTINGS: TRenderAmbientOcclusionGtaoSettings = {
  slices: 3,
  steps: 3,
};

/** Four rays a point. */
export const DEFAULT_RENDER_AMBIENT_OCCLUSION_RTAO_SETTINGS: TRenderAmbientOcclusionRtaoSettings = {
  rays: 4,
};

/** XeGTAO's defaults, at a metre, and VBAO's and RTAO's searches for when they search. */
export const DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS: TRenderAmbientOcclusionSettings = {
  gtao: DEFAULT_RENDER_AMBIENT_OCCLUSION_GTAO_SETTINGS,
  isEnabled: true,
  method: ERenderAmbientOcclusionMethod.GTAO,
  radius: 1,
  rtao: DEFAULT_RENDER_AMBIENT_OCCLUSION_RTAO_SETTINGS,
  strength: 1,
  vbao: DEFAULT_RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS,
};

/** OpenXRay's own exposure (`xrRender_console.cpp`). */
export const DEFAULT_RENDER_EXPOSURE_SETTINGS: TRenderExposureSettings = {
  adaptation: 1,
  amount: 0.7,
  compensation: 0,
  isEnabled: true,
  lowLuminance: 0.0001,
  middleGray: 1,
};

/**
 * The engine's own fog, with the enhanced fog's strengths but no scattering for when it is drawn, and at the anchored
 * light levels half the weather's colour and half the sky's.
 */
export const DEFAULT_RENDER_FOG_SETTINGS: TRenderFogSettings = {
  colour: ERenderFogColour.BLEND,
  density: 0.4,
  height: 8,
  skyFade: 0.75,
  mode: ERenderFogMode.ENGINE,
  scattering: 0,
  sunColor: 0.1,
};

/** The sky as drawn, with the debanding's shipped strengths for when it is debanded: two passes out to 48 pixels. */
export const DEFAULT_RENDER_DEBANDING_SETTINGS: TRenderDebandingSettings = {
  mode: ERenderDebandingMode.ENGINE,
  passes: 2,
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

/** The physical light's frame ungraded. */
export const DEFAULT_RENDER_GRADING_SETTINGS: TRenderGradingSettings = {
  contrast: 1,
  saturation: 1,
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
  isShadowed: false,
  radius: 49,
  shadowCascades: 0,
  shadowRadius: 50,
  slopeLimit: 90,
};

/** The engine's own: no bounced light, and all of it from three metres around for when it is gathered. */
export const DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS: TRenderIndirectLightSettings = {
  intensity: 1,
  mode: ERenderIndirectLightMode.ENGINE,
  radius: 3,
  steps: 2,
};

/**
 * The engine's streaks and own wetting; enhanced, half the puddle sites filled to 300 m, each taking up to 40%
 * reflection; the refracting drops over 30 m about the camera, their rings out to 75 m.
 */
export const DEFAULT_RENDER_RAIN_SETTINGS: TRenderRainSettings = {
  distance: 300,
  dropSquare: 30,
  drops: ERenderRainDropsMode.ENGINE,
  isReliefFlooded: true,
  mode: ERenderRainMode.ENGINE,
  pace: 1,
  puddles: 0.5,
  reflectivity: 0.4,
  ringDistance: 75,
  ringSize: 1,
  ringStrength: 1,
  ripples: 1,
};

/**
 * The engine's own: the surfaces reflect the cube alone and the water the screen. For when the surfaces are traced,
 * rays of 150 metres at half size over 96 cells at full strength over ground wet to 300 metres, budgeted, the sky's
 * highlights a quarter brighter as their light; the water marches 16 steps; a missed pixel is traced every other frame.
 */
export const DEFAULT_RENDER_REFLECTION_SETTINGS: TRenderReflectionSettings = {
  distance: 150,
  reach: 300,
  intensity: 1,
  mode: ERenderReflectionMode.ENGINE,
  probe: ERenderReflectionProbe.FINE,
  traceSize: ERenderTraceSize.HALF,
  steps: 96,
  skyGain: 1.25,
  isGrassReflected: false,
  traceRoughness: 0.6,
  isDryTraced: false,
  isRayBudgeted: true,
  isWaterReflected: true,
  waterSteps: 16,
  isTraced: false,
  traceInterval: 2,
};

/**
 * The engine's own: every spawned light, and none of the level file's; no shafts until they cost less; at the
 * anchored light level falling off as light does in the world.
 */
export const DEFAULT_RENDER_LIGHTS_SETTINGS: TRenderLightsSettings = {
  falloff: ERenderLightFalloff.PHYSICAL,
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
 * strengths for when it is drawn: eight steps within 60 m, 0.02 of the detail deep, refined, self-shadowed, its layers
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
  steps: 8,
  surfaceHeight: 0.02,
  surfaceRange: 60,
};

/**
 * The engines' own: the terrain's details repeating as a grid; and for when they are varied, hexagons about a seventh
 * of a detail's repeat across, upright, meeting as the hex tiling demo blends them.
 */
export const DEFAULT_RENDER_TERRAIN_SETTINGS: TRenderTerrainSettings = {
  rotation: 0,
  scale: 2,
  sharpness: 7,
  tiling: ERenderTerrainTiling.ENGINE,
};

/**
 * Every camera effect off, and for when each is on: film's 180° shutter blurring at most 4% of the frame's height, a
 * full-frame sensor at f/2.8 focused at 10 m, half the lens's falloff and colour split, and a light, fine grain.
 */
export const DEFAULT_RENDER_CAMERA_SETTINGS: TRenderCameraSettings = {
  aberrationStrength: 0.5,
  fStop: 2.8,
  focusDistance: 10,
  grainSize: 1.6,
  grainStrength: 0.3,
  isAberrated: false,
  isDefocused: false,
  isGrained: false,
  isMotionBlurred: false,
  isVignetted: false,
  motionLimit: 0.04,
  sensorHeight: 24,
  shutter: 0.5,
  vignetteStrength: 0.5,
};

/** The engine's own: no contact shadows, and their strengths for when they are drawn, towards four lights a pixel. */
export const DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS: TRenderContactShadowSettings = {
  intensity: 1,
  isFullResolution: false,
  length: 0.6,
  lights: 4,
  mode: ERenderContactShadowMode.ENGINE,
  steps: 16,
  thickness: 0.1,
};

/**
 * Every cascade, the engine's three and the far one, each blended into the next over its last quarter, a filter two
 * texels wide, so swaying grass does not shimmer in the nearest, and no contact shadows.
 */
export const DEFAULT_RENDER_SHADOW_SETTINGS: TRenderShadowSettings = {
  bias: 1.5,
  blend: 0.25,
  cascades: RENDER_SHADOW_CASCADE_WIDTHS,
  contact: DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS,
  filter: 2,
  isEnabled: true,
  isStaggered: true,
  resolution: 2048,
  swayCascades: 4,
  swayRate: 60,
};

/** Drawn at the viewport's size; sharpened a tenth once upscaled; DLSS's models NVIDIA's pick, reconstructing no rays. */
export const DEFAULT_RENDER_UPSCALING_SETTINGS: TRenderUpscalingSettings = {
  dlssPreset: ERenderDlssPreset.DEFAULT,
  isRayReconstructed: false,
  rayReconstructionPreset: ERenderRayReconstructionPreset.DEFAULT,
  scale: ERenderScale.NATIVE,
  sharpening: 0.1,
};

/** The engine's own shafts, with the froxel volume at its designed strength and a quarter's lean, to 120 m. */
export const DEFAULT_RENDER_VOLUMETRIC_SETTINGS: TRenderVolumetricSettings = {
  anisotropy: 0.25,
  distance: 120,
  intensity: 1,
  mode: ERenderVolumetricMode.ENGINE,
};

/**
 * The enhanced water's designed strengths, but for caustics a sixth as bright, calm water flowing a fifth as fast, and
 * its maps' repeat broken.
 */
const DEFAULT_RENDER_ENHANCED_WATER_SETTINGS: TRenderEnhancedWaterSettings = {
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
  waveHeight: 0.017,
  waveSpeed: 25,
};
