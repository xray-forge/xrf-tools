import { DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS } from "#/contract/renderer-ambient-occlusion-settings";
import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { DEFAULT_RENDERER_EXPOSURE_SETTINGS } from "#/contract/renderer-exposure-settings";
import { IRendererFeatureSettings } from "#/contract/renderer-feature-settings";
import { DEFAULT_RENDERER_GRASS_SETTINGS } from "#/contract/renderer-grass-settings";
import { DEFAULT_RENDERER_LIGHTS_SETTINGS } from "#/contract/renderer-lights-settings";
import { DEFAULT_RENDERER_LOD_SETTINGS } from "#/contract/renderer-lod-settings";
import { DEFAULT_RENDERER_SHADOW_SETTINGS } from "#/contract/renderer-shadow-settings";
import { DEFAULT_RENDERER_UPSCALING_SETTINGS } from "#/contract/renderer-upscaling-settings";
import { DEFAULT_RENDERER_WATER_SETTINGS } from "#/contract/renderer-water-settings";

/**
 * The named sets of features: `Base` the engine's defaults, `Editing` responsiveness before looks for an editor.
 */
export enum ERendererPreset {
  BASE = "base",
  EDITING = "editing",
}

/** What each preset sets every feature to. */
export const RENDERER_PRESETS: Readonly<Record<ERendererPreset, IRendererFeatureSettings>> = {
  [ERendererPreset.BASE]: {
    ambientOcclusion: DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS,
    antialiasing: ERendererAntialiasing.SMAA,
    exposure: DEFAULT_RENDERER_EXPOSURE_SETTINGS,
    grass: DEFAULT_RENDERER_GRASS_SETTINGS,
    isOcclusionCulled: true,
    lights: DEFAULT_RENDERER_LIGHTS_SETTINGS,
    lod: DEFAULT_RENDERER_LOD_SETTINGS,
    shadows: DEFAULT_RENDERER_SHADOW_SETTINGS,
    upscaling: DEFAULT_RENDERER_UPSCALING_SETTINGS,
    water: DEFAULT_RENDERER_WATER_SETTINGS,
  },
  [ERendererPreset.EDITING]: {
    ambientOcclusion: { ...DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS, isEnabled: false },
    antialiasing: ERendererAntialiasing.NONE,
    exposure: DEFAULT_RENDERER_EXPOSURE_SETTINGS,
    grass: { ...DEFAULT_RENDERER_GRASS_SETTINGS, isEnabled: false },
    isOcclusionCulled: true,
    // Unshadowed, the lights cost a pass over the screen: an editor keeps seeing what lights a room.
    lights: { ...DEFAULT_RENDERER_LIGHTS_SETTINGS, isShadowed: false },
    lod: DEFAULT_RENDERER_LOD_SETTINGS,
    shadows: { ...DEFAULT_RENDERER_SHADOW_SETTINGS, isEnabled: false },
    upscaling: DEFAULT_RENDERER_UPSCALING_SETTINGS,
    water: DEFAULT_RENDERER_WATER_SETTINGS,
  },
};
