import {
  ERenderAmbientOcclusionMethod,
  ERenderAntialiasing,
  ERenderContactShadowMode,
  ERenderFoliageMode,
  ERenderIndirectLightMode,
  ERenderRainMode,
  ERenderReflectionMode,
  ERenderWaterMode,
} from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDER_CONTACT_SHADOW_SETTINGS,
  DEFAULT_RENDER_EXPOSURE_SETTINGS,
  DEFAULT_RENDER_FOLIAGE_SETTINGS,
  DEFAULT_RENDER_GRASS_SETTINGS,
  DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
  DEFAULT_RENDER_LIGHTS_SETTINGS,
  DEFAULT_RENDER_LOD_SETTINGS,
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
    exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS,
    // The application's own choice over the engine's, as the antialiasing is: the enhanced foliage motion.
    grass: {
      ...DEFAULT_RENDER_GRASS_SETTINGS,
      foliage: { ...DEFAULT_RENDER_FOLIAGE_SETTINGS, mode: ERenderFoliageMode.ENHANCED },
    },
    // The application's own choice over the engine's, as the antialiasing is: the light bounced in the occlusion's
    // own search.
    indirectLight: { ...DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS, mode: ERenderIndirectLightMode.ENHANCED },
    isOcclusionCulled: true,
    lights: DEFAULT_RENDER_LIGHTS_SETTINGS,
    lod: DEFAULT_RENDER_LOD_SETTINGS,
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
    upscaling: DEFAULT_RENDER_UPSCALING_SETTINGS,
    // The application's own choice over the engine's, as the antialiasing is: the enhanced water.
    water: { ...DEFAULT_RENDER_WATER_SETTINGS, mode: ERenderWaterMode.ENHANCED },
  },
  [ERenderPreset.EDITING]: {
    ambientOcclusion: { ...DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS, isEnabled: false },
    antialiasing: ERenderAntialiasing.NONE,
    exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS,
    grass: { ...DEFAULT_RENDER_GRASS_SETTINGS, isEnabled: false },
    indirectLight: DEFAULT_RENDER_INDIRECT_LIGHT_SETTINGS,
    isOcclusionCulled: true,
    // Unshadowed, the lights cost a pass over the screen: an editor keeps seeing what lights a room.
    lights: { ...DEFAULT_RENDER_LIGHTS_SETTINGS, isShadowed: false },
    lod: DEFAULT_RENDER_LOD_SETTINGS,
    rain: DEFAULT_RENDER_RAIN_SETTINGS,
    reflections: DEFAULT_RENDER_REFLECTION_SETTINGS,
    shadows: { ...DEFAULT_RENDER_SHADOW_SETTINGS, isEnabled: false },
    upscaling: DEFAULT_RENDER_UPSCALING_SETTINGS,
    water: DEFAULT_RENDER_WATER_SETTINGS,
  },
};
