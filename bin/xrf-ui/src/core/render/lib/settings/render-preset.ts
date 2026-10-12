import {
  ERenderAmbientOcclusionMethod,
  ERenderAntialiasing,
  ERenderBloomMode,
  ERenderContactShadowMode,
  ERenderDebandingMode,
  ERenderFogMode,
  ERenderFoliageMode,
  ERenderGlassMode,
  ERenderIndirectLightMode,
  ERenderLightShadowFilter,
  ERenderParallaxMode,
  ERenderRainMode,
  ERenderReflectionMode,
  ERenderTextureFiltering,
  ERenderWaterMode,
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
  DEFAULT_RENDER_WATER_SETTINGS,
} from "@/core/render/lib/settings/render-feature-defaults";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

/**
 * The named sets of features: `Base` the engine's defaults, smoothed by TAA, `Editing` responsiveness before looks for
 * an editor.
 */
export enum ERenderPreset {
  BASE = "base",
  EDITING = "editing",
}

/** What each preset sets every feature to. */
export const RENDER_PRESETS: Readonly<Record<ERenderPreset, IRenderFeatureSettings>> = {
  [ERenderPreset.BASE]: {
    // The application's own choice over XeGTAO's, as the antialiasing is: VBAO, which lets light
    // through behind grass and fences.
    ambientOcclusion: {
      ...DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
      method: ERenderAmbientOcclusionMethod.VBAO,
    },
    // The application's own choice over the engine's: the resolve the cut-outs' hashed alpha and the jitter are for.
    antialiasing: ERenderAntialiasing.TAA,
    // The application's own choice over the engine's, as the antialiasing is: the sky's bands smoothed, which reads
    // the sky's own pixels alone and leaves the rest of the frame as drawn.
    debanding: { ...DEFAULT_RENDER_DEBANDING_SETTINGS, mode: ERenderDebandingMode.ENHANCED },
    // The application's own choice over the engine's (owner, 2026-10-11): the enhanced bloom.
    enhancedBloom: { ...DEFAULT_RENDER_ENHANCED_BLOOM_SETTINGS, mode: ERenderBloomMode.ENHANCED },
    exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS,
    // The application's own choice over the engine's, as the antialiasing is: the fog thickened low down and the
    // frame's bright parts scattered into it, at four small passes.
    fog: { ...DEFAULT_RENDER_FOG_SETTINGS, mode: ERenderFogMode.ENHANCED },
    // The application's own choice over the engine's, as the antialiasing is: glass refracting what is behind it.
    glass: { ...DEFAULT_RENDER_GLASS_SETTINGS, mode: ERenderGlassMode.ENHANCED },
    // The application's own choice over the engine's, as the antialiasing is: the enhanced foliage motion, each tuft
    // stood along its ground's slope.
    grass: {
      ...DEFAULT_RENDER_GRASS_SETTINGS,
      foliage: { ...DEFAULT_RENDER_FOLIAGE_SETTINGS, mode: ERenderFoliageMode.ENHANCED },
      isAligned: true,
      // Planted out to 150 m rather than the engine's 49.
      radius: 150,
    },
    // The application's own choice over the engine's, as the antialiasing is: the light bounced in the occlusion's
    // own search.
    indirectLight: { ...DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS, mode: ERenderIndirectLightMode.ENHANCED },
    isOcclusionCulled: true,
    // The application's own choice over the engine's, as the antialiasing is: Anomaly's soft shadows for the lamps.
    lights: { ...DEFAULT_RENDER_LIGHTS_SETTINGS, shadowFilter: ERenderLightShadowFilter.SOFT },
    // Trees stand in as impostors twice as far as the engine's: a clump's screen area falls with its distance
    // squared and the thresholds are squared, so half the engine's `r_ssaLOD_A` and `r_ssaLOD_B`.
    lod: {
      ...DEFAULT_RENDER_LOD_SETTINGS,
      ssaA: DEFAULT_RENDER_LOD_SETTINGS.ssaA / 2,
      ssaB: DEFAULT_RENDER_LOD_SETTINGS.ssaB / 2,
    },
    // The application's own choice over the engine's, as the antialiasing is: the terrain marched over its details'
    // heights.
    parallax: { ...DEFAULT_RENDER_PARALLAX_SETTINGS, mode: ERenderParallaxMode.ENHANCED },
    // The application's own choice over the engine's, as the antialiasing is: wet surfaces that build up and dry,
    // and puddles.
    rain: { ...DEFAULT_RENDER_RAIN_SETTINGS, mode: ERenderRainMode.ENHANCED },
    // The application's own choice over the engine's, as the antialiasing is: what the frame shows reflected where a
    // ray meets it, the cube elsewhere.
    reflections: { ...DEFAULT_RENDER_REFLECTION_SETTINGS, mode: ERenderReflectionMode.ENHANCED },
    // The application's own choice over the engine's, as the antialiasing is: contact shadows under the cascades.
    shadows: {
      ...DEFAULT_RENDER_SHADOW_SETTINGS,
      contact: { ...DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS, mode: ERenderContactShadowMode.ENHANCED },
    },
    textureFiltering: ERenderTextureFiltering.ANISOTROPIC16,
    // The engine's full quality, `texture_lod 0`, as Anomaly's extreme preset sets it.
    textureLod: 0,
    upscaling: DEFAULT_RENDER_UPSCALING_SETTINGS,
    // The application's own choice over the engine's, as the antialiasing is: the enhanced water.
    water: { ...DEFAULT_RENDER_WATER_SETTINGS, mode: ERenderWaterMode.ENHANCED },
  },
  [ERenderPreset.EDITING]: {
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
    upscaling: DEFAULT_RENDER_UPSCALING_SETTINGS,
    water: DEFAULT_RENDER_WATER_SETTINGS,
  },
};
