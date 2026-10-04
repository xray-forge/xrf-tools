import { ERenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDER_EXPOSURE_SETTINGS,
  DEFAULT_RENDER_GRASS_SETTINGS,
  DEFAULT_RENDER_LIGHTS_SETTINGS,
  DEFAULT_RENDER_LOD_SETTINGS,
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
    ambientOcclusion: DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS,
    // The application's own choice over the engine's: the resolve the cut-outs' hashed alpha and the jitter are for.
    antialiasing: ERenderAntialiasing.TAA,
    exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS,
    grass: DEFAULT_RENDER_GRASS_SETTINGS,
    isOcclusionCulled: true,
    lights: DEFAULT_RENDER_LIGHTS_SETTINGS,
    lod: DEFAULT_RENDER_LOD_SETTINGS,
    shadows: DEFAULT_RENDER_SHADOW_SETTINGS,
    upscaling: DEFAULT_RENDER_UPSCALING_SETTINGS,
    water: DEFAULT_RENDER_WATER_SETTINGS,
  },
  [ERenderPreset.EDITING]: {
    ambientOcclusion: { ...DEFAULT_RENDER_AMBIENT_OCCLUSION_SETTINGS, isEnabled: false },
    antialiasing: ERenderAntialiasing.NONE,
    exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS,
    grass: { ...DEFAULT_RENDER_GRASS_SETTINGS, isEnabled: false },
    isOcclusionCulled: true,
    // Unshadowed, the lights cost a pass over the screen: an editor keeps seeing what lights a room.
    lights: { ...DEFAULT_RENDER_LIGHTS_SETTINGS, isShadowed: false },
    lod: DEFAULT_RENDER_LOD_SETTINGS,
    shadows: { ...DEFAULT_RENDER_SHADOW_SETTINGS, isEnabled: false },
    upscaling: DEFAULT_RENDER_UPSCALING_SETTINGS,
    water: DEFAULT_RENDER_WATER_SETTINGS,
  },
};
