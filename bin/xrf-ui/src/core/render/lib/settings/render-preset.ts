import {
  ERenderAmbientOcclusionMethod,
  ERenderAmbientOcclusionQuality,
  ERenderAntialiasing,
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
  ERenderRainDropsMode,
  ERenderRainMode,
  ERenderReflectionMode,
  ERenderReflectionProbe,
  ERenderReflectionQuality,
  ERenderReflectionTraceQuality,
  ERenderTextureFiltering,
  ERenderTonemapMode,
  ERenderVolumetricMode,
  ERenderWaterMode,
  RENDER_MAX_CONTACT_SHADOW_LIGHTS,
  RENDER_MAX_SHADOW_FILTER,
} from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS,
  DEFAULT_RENDER_DEBANDING_SETTINGS,
  DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS,
  DEFAULT_RENDER_EXPOSURE_SETTINGS,
  DEFAULT_RENDER_FOG_SETTINGS,
  DEFAULT_RENDER_FOLIAGE_SETTINGS,
  DEFAULT_RENDER_GLASS_SETTINGS,
  DEFAULT_RENDER_GRASS_SETTINGS,
  DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
  DEFAULT_RENDER_LIGHTS_SETTINGS,
  DEFAULT_RENDER_LOD_SETTINGS,
  DEFAULT_RENDER_PARALLAX_SETTINGS,
  DEFAULT_RENDER_RAIN_SETTINGS,
  DEFAULT_RENDER_REFLECTION_SETTINGS,
  DEFAULT_RENDER_SHADOW_SETTINGS,
  DEFAULT_RENDER_UPSCALING_SETTINGS,
  DEFAULT_RENDER_VOLUMETRIC_SETTINGS,
  DEFAULT_RENDER_WATER_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

/**
 * The named sets of features: `Editing` responsiveness before looks for the editing tools, `Base` the application's
 * look with every Enhanced mode at a cheap quality, `Enhanced` the same look at its best, short of ray reconstruction
 * and traced occlusion.
 */
export enum ERenderPreset {
  EDITING = "editing",
  BASE = "base",
  ENHANCED = "enhanced",
}

/** The application's look: the Enhanced modes over the engine's, each at a quality a modest GPU holds. */
const BASE_PRESET: IRenderFeatureSettings = {
  // VBAO over XeGTAO, which lets light through behind grass and fences.
  ambientOcclusion: {
    ...DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
    method: ERenderAmbientOcclusionMethod.VBAO,
    quality: ERenderAmbientOcclusionQuality.MEDIUM,
  },
  // The resolve the cut-outs' hashed alpha and the jitter are for.
  antialiasing: ERenderAntialiasing.TAA,
  // The sky's bands smoothed, which reads the sky's own pixels alone and leaves the rest of the frame as drawn.
  debanding: {
    ...DEFAULT_RENDER_DEBANDING_SETTINGS,
    mode: ERenderDebandingMode.ENHANCED,
    quality: ERenderDebandingQuality.LOW,
  },
  enhancedBloom: { ...DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS, mode: ERenderBloomMode.ENHANCED },
  exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS,
  // The fog thickened low down and the frame's bright parts scattered into it, at four small passes.
  fog: { ...DEFAULT_RENDER_FOG_SETTINGS, mode: ERenderFogMode.ENHANCED },
  // Glass refracting what is behind it.
  glass: { ...DEFAULT_RENDER_GLASS_SETTINGS, mode: ERenderGlassMode.ENHANCED },
  // The enhanced foliage motion, each tuft stood along its ground's slope, planted out to 150 m rather than the
  // engine's 49 (nearer, its edge shows on the slopes ahead) and shadowed near the camera.
  grass: {
    ...DEFAULT_RENDER_GRASS_SETTINGS,
    foliage: { ...DEFAULT_RENDER_FOLIAGE_SETTINGS, mode: ERenderFoliageMode.ENHANCED },
    isAligned: true,
    isShadowed: true,
    radius: 150,
    shadowRadius: 25,
  },
  // The light bounced in the occlusion's own search.
  indirectLight: { ...DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS, mode: ERenderIndirectLightMode.ENHANCED },
  isOcclusionCulled: true,
  // Anomaly's soft shadows for the lamps.
  lights: { ...DEFAULT_RENDER_LIGHTS_SETTINGS, shadowFilter: ERenderLightShadowFilter.SOFT },
  // Trees stand in as impostors twice as far as the engine's: a clump's screen area falls with its distance squared
  // and the thresholds are squared, so half the engine's `r_ssaLOD_A` and `r_ssaLOD_B`.
  lod: {
    ...DEFAULT_RENDER_LOD_SETTINGS,
    ssaA: DEFAULT_RENDER_LOD_SETTINGS.ssaA / 2,
    ssaB: DEFAULT_RENDER_LOD_SETTINGS.ssaB / 2,
  },
  // The terrain marched over its details' heights.
  parallax: { ...DEFAULT_RENDER_PARALLAX_SETTINGS, mode: ERenderParallaxMode.ENHANCED },
  // Wet surfaces that build up and dry, and puddles, under the engine's streaks.
  rain: { ...DEFAULT_RENDER_RAIN_SETTINGS, mode: ERenderRainMode.ENHANCED },
  // What the frame shows reflected where a ray meets it, the cube elsewhere.
  reflections: {
    ...DEFAULT_RENDER_REFLECTION_SETTINGS,
    mode: ERenderReflectionMode.ENHANCED,
    quality: ERenderReflectionQuality.MEDIUM,
  },
  // Contact shadows under the cascades, at the frame's size, so thin foliage does not flicker under a temporal resolve.
  shadows: {
    ...DEFAULT_RENDER_SHADOW_SETTINGS,
    contact: {
      ...DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS,
      isFullResolution: true,
      mode: ERenderContactShadowMode.ENHANCED,
    },
  },
  textureFiltering: ERenderTextureFiltering.ANISOTROPIC16,
  // The engine's full quality, `texture_lod 0`, as Anomaly's extreme preset sets it.
  textureLod: 0,
  // The scene held as linear light and tonemapped once at the viewport's size, as an upscaler wants it.
  tonemap: ERenderTonemapMode.ENHANCED,
  upscaling: DEFAULT_RENDER_UPSCALING_SETTINGS,
  // The sun and every lamp scattered in one froxel volume, in the engine's shafts' place, out to 80 m.
  volumetric: { ...DEFAULT_RENDER_VOLUMETRIC_SETTINGS, distance: 80, mode: ERenderVolumetricMode.ENHANCED },
  water: { ...DEFAULT_RENDER_WATER_SETTINGS, mode: ERenderWaterMode.ENHANCED },
};

/** The application's look at its best: every quality at its top step, short of ray reconstruction and traced occlusion. */
const ENHANCED_PRESET: IRenderFeatureSettings = {
  ...BASE_PRESET,
  ambientOcclusion: { ...BASE_PRESET.ambientOcclusion, quality: ERenderAmbientOcclusionQuality.ULTRA },
  debanding: { ...BASE_PRESET.debanding, quality: ERenderDebandingQuality.ULTRA },
  grass: { ...BASE_PRESET.grass, radius: 300, shadowRadius: 100 },
  parallax: { ...BASE_PRESET.parallax, steps: 32 },
  // Drops bending the scene behind them, their landings ringing the water and the puddles.
  rain: { ...BASE_PRESET.rain, drops: ERenderRainDropsMode.ENHANCED },
  // Traced where the GPU traces rays, the grass reflected too.
  reflections: {
    ...BASE_PRESET.reflections,
    isGrassReflected: true,
    isTraced: true,
    probe: ERenderReflectionProbe.FINEST,
    quality: ERenderReflectionQuality.ULTRA,
    traceQuality: ERenderReflectionTraceQuality.ULTRA,
  },
  shadows: {
    ...BASE_PRESET.shadows,
    contact: { ...BASE_PRESET.shadows.contact, lights: RENDER_MAX_CONTACT_SHADOW_LIGHTS, steps: 32 },
    filter: RENDER_MAX_SHADOW_FILTER,
    resolution: 4096,
  },
  volumetric: { ...BASE_PRESET.volumetric, distance: 200 },
};

/** Responsiveness before looks for the editing tools: the engine's modes, the costly passes off. */
const EDITING_PRESET: IRenderFeatureSettings = {
  ambientOcclusion: { ...DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS, isEnabled: false },
  antialiasing: ERenderAntialiasing.NONE,
  debanding: DEFAULT_RENDER_DEBANDING_SETTINGS,
  enhancedBloom: DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS,
  exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS,
  fog: DEFAULT_RENDER_FOG_SETTINGS,
  glass: DEFAULT_RENDER_GLASS_SETTINGS,
  grass: { ...DEFAULT_RENDER_GRASS_SETTINGS, isEnabled: false },
  indirectLight: DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
  isOcclusionCulled: true,
  // Unshadowed, the lights cost a pass over the screen: an editor keeps seeing what lights a room.
  lights: { ...DEFAULT_RENDER_LIGHTS_SETTINGS, isShadowed: false },
  lod: DEFAULT_RENDER_LOD_SETTINGS,
  parallax: DEFAULT_RENDER_PARALLAX_SETTINGS,
  rain: DEFAULT_RENDER_RAIN_SETTINGS,
  reflections: DEFAULT_RENDER_REFLECTION_SETTINGS,
  shadows: { ...DEFAULT_RENDER_SHADOW_SETTINGS, isEnabled: false },
  textureFiltering: ERenderTextureFiltering.ANISOTROPIC16,
  // The engine's full quality, `texture_lod 0`, as Anomaly's extreme preset sets it.
  textureLod: 0,
  tonemap: ERenderTonemapMode.ENGINE,
  upscaling: DEFAULT_RENDER_UPSCALING_SETTINGS,
  volumetric: DEFAULT_RENDER_VOLUMETRIC_SETTINGS,
  water: DEFAULT_RENDER_WATER_SETTINGS,
};

/** What each preset sets every feature to. */
export const RENDER_PRESETS: Readonly<Record<ERenderPreset, IRenderFeatureSettings>> = {
  [ERenderPreset.EDITING]: EDITING_PRESET,
  [ERenderPreset.BASE]: BASE_PRESET,
  [ERenderPreset.ENHANCED]: ENHANCED_PRESET,
};
