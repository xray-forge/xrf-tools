import { Nullable } from "@xrf/types";

import { ERenderAntialiasing, RenderAntialiasing, RenderTextureFiltering } from "@/core/ipc/types/xrf-renderer";
import { toRenderFeatureChoice } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import {
  IRenderFeatureSettings,
  TRenderAmbientOcclusionSettings,
  TRenderDebandingSettings,
  TRenderEnhancedBloomSettings,
  TRenderFogSettings,
  TRenderGlassSettings,
  TRenderGrassSettings,
  TRenderIndirectLightSettings,
  TRenderLightsSettings,
  TRenderParallaxSettings,
  TRenderRainSettings,
  TRenderReflectionSettings,
  TRenderShadowSettings,
  TRenderUpscalingSettings,
  TRenderVolumetricSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/render-feature-settings";
import {
  ERenderResolution,
  RENDER_OUTPUT_SCALES,
  RENDER_RESOLUTIONS,
} from "@/core/render/lib/settings/render-resolution";

/** The shadow settings a level view may set for itself. */
export type TLevelShadowOptions = Pick<
  TRenderShadowSettings,
  "bias" | "blend" | "cascades" | "contact" | "filter" | "resolution"
>;

/** The ambient occlusion settings a level view may set for itself. */
export type TLevelAmbientOcclusionOptions = Pick<
  TRenderAmbientOcclusionSettings,
  "method" | "quality" | "radius" | "strength" | "vbao"
>;

/** The indirect light settings a level view may set for itself: all of them. */
export type TLevelIndirectLightOptions = TRenderIndirectLightSettings;

/** The reflection settings a level view may set for itself: all of them. */
export type TLevelReflectionOptions = TRenderReflectionSettings;

/** The rain settings a level view may set for itself: all of them. */
export type TLevelRainOptions = TRenderRainSettings;

/** The fog settings a level view may set for itself: all of them. */
export type TLevelFogOptions = TRenderFogSettings;

/** The sky debanding settings a level view may set for itself: all of them. */
export type TLevelDebandingOptions = TRenderDebandingSettings;

/** The glass settings a level view may set for itself: all of them. */
export type TLevelGlassOptions = TRenderGlassSettings;

/** The volumetric lighting settings a level view may set for itself: all of them. */
export type TLevelVolumetricOptions = TRenderVolumetricSettings;

/** The parallax settings a level view may set for itself: all of them; turning it off is the shading's toggle. */
export type TLevelParallaxOptions = TRenderParallaxSettings;

/** The bloom's mode and the enhanced bloom's settings a level view may set for itself: all of them. */
export type TLevelEnhancedBloomOptions = TRenderEnhancedBloomSettings;

/** The grass settings a level view may set for itself. */
export type TLevelGrassOptions = Pick<
  TRenderGrassSettings,
  "density" | "foliage" | "height" | "isAligned" | "isShadowed" | "radius" | "shadowRadius" | "slopeLimit"
>;

/** The water settings a level view may set for itself: all but whether it is drawn, which the toolbar toggles. */
export type TLevelWaterOptions = Omit<TRenderWaterSettings, "isEnabled">;

/** The lights settings a level view may set for itself. */
export type TLevelLightsOptions = Pick<
  TRenderLightsSettings,
  "isLevelLights" | "isShadowed" | "isVolumetric" | "shadowFilter"
>;

/**
 * What a level view sets over the renderer's settings for itself: whatever it leaves unset follows them.
 */
export interface ILevelFeatureOptions {
  ambientOcclusion: Partial<TLevelAmbientOcclusionOptions>;
  grass: Partial<TLevelGrassOptions>;
  /** The indirect light, which no toolbar toggle turns off: its own mode does. */
  indirectLight: Partial<TLevelIndirectLightOptions>;
  lights: Partial<TLevelLightsOptions>;
  /** The reflections, which no toolbar toggle turns off: their own mode does. */
  reflections: Partial<TLevelReflectionOptions>;
  /** The rain's wetting, which no toolbar toggle turns off: its own mode does. */
  rain: Partial<TLevelRainOptions>;
  /** How the fog is drawn beyond the weather's keys, which the fog's toggle does not turn: its own mode does. */
  fog: Partial<TLevelFogOptions>;
  /** The sky debanding, which no toolbar toggle turns off: its own mode does. */
  debanding: Partial<TLevelDebandingOptions>;
  /** How glass is drawn, which no toolbar toggle turns off: its own mode does. */
  glass: Partial<TLevelGlassOptions>;
  /** How the light the air scatters is drawn, which no toolbar toggle turns off: its own mode does. */
  volumetric: Partial<TLevelVolumetricOptions>;
  /** How the surfaces are displaced by their height, which the shading's toggle turns off whole. */
  parallax: Partial<TLevelParallaxOptions>;
  /** Which bloom draws and the enhanced bloom's strengths, which no toolbar toggle turns off: its own mode does. */
  enhancedBloom: Partial<TLevelEnhancedBloomOptions>;
  /** The mode edges are smoothed with while the settings smooth them at all, or null for the settings' own. */
  antialiasing: Nullable<RenderAntialiasing>;
  /** How the view's surfaces sample their textures, or null for the settings' own. */
  textureFiltering: Nullable<RenderTextureFiltering>;
  /** How much smaller than the viewport the view is drawn, where it sets its own. */
  upscaling: Partial<TRenderUpscalingSettings>;
  /** What the view resolves its frame at, where it sets its own: a resolution, and the scale an automatic one takes. */
  output: TLevelOutputOptions;
  shadows: Partial<TLevelShadowOptions>;
  water: Partial<TLevelWaterOptions>;
}

/** What a view resolves its frame at where it sets its own, over the settings' resolution and scale. */
export type TLevelOutputOptions = Partial<{ resolution: ERenderResolution; scale: number }>;

/**
 * @param stored - What was stored for a view's features.
 * @returns The output it set, each part dropped where it is not one the settings offer.
 */
function toLevelOutputOptions(stored: unknown): TLevelOutputOptions {
  const output: Record<string, unknown> =
    (stored as { output?: Record<string, unknown> } | null | undefined)?.output ?? {};
  const resolution: ERenderResolution | undefined = RENDER_RESOLUTIONS.find((it) => it === output.resolution);
  const scale: unknown = output.scale;
  const isScale: boolean =
    typeof scale === "number" && scale >= RENDER_OUTPUT_SCALES.min && scale <= RENDER_OUTPUT_SCALES.max;

  return {
    ...(resolution ? { resolution } : {}),
    ...(isScale ? { scale: scale as number } : {}),
  };
}

/** The modes a view picks from, which smooth something: turning it off is the toggle's. */
export const LEVEL_ANTIALIASING_MODES: ReadonlyArray<RenderAntialiasing> = Object.values(ERenderAntialiasing).filter(
  (mode: RenderAntialiasing) => mode !== ERenderAntialiasing.NONE
);

/** Each group's settings a view may set, which is all a stored view keeps. */
const LEVEL_FEATURE_KEYS: {
  readonly [K in TLevelFeatureKey]: ReadonlyArray<keyof ILevelFeatureOptions[K]>;
} = {
  ambientOcclusion: ["method", "quality", "radius", "strength", "vbao"],
  grass: ["density", "foliage", "height", "isAligned", "isShadowed", "radius", "shadowRadius", "slopeLimit"],
  lights: ["isLevelLights", "isShadowed", "isVolumetric", "shadowFilter"],
  shadows: ["bias", "blend", "cascades", "contact", "filter", "resolution"],
  water: ["distortion", "enhanced", "isDistorted", "isSoft", "mode", "reflection", "ripple", "waveHeight", "waveSpeed"],
};

/** The groups no toolbar toggle turns off, whose own mode does, of which a view may set every setting. */
export type TLevelModeFeatureKey =
  | "debanding"
  | "enhancedBloom"
  | "fog"
  | "glass"
  | "indirectLight"
  | "parallax"
  | "rain"
  | "reflections"
  | "volumetric";

/** Each such group's settings a view may set. */
const LEVEL_MODE_FEATURE_KEYS: {
  readonly [K in TLevelModeFeatureKey]: ReadonlyArray<keyof ILevelFeatureOptions[K]>;
} = {
  debanding: ["mode", "quality", "radius"],
  enhancedBloom: ["blur", "exposure", "mode", "sky", "threshold", "vibrance"],
  fog: ["density", "height", "mode", "scattering", "skyFade", "sunColor"],
  glass: ["mode", "refraction"],
  indirectLight: ["intensity", "mode", "radius"],
  parallax: [
    "blending",
    "cavity",
    "height",
    "isRefined",
    "isSteep",
    "mode",
    "range",
    "shadow",
    "steps",
    "surfaceHeight",
    "surfaceRange",
  ],
  rain: ["distance", "isReliefFlooded", "mode", "puddles", "reflectivity", "ripples"],
  reflections: ["distance", "intensity", "mode", "quality"],
  volumetric: ["mode", "intensity", "anisotropy", "distance"],
};

/**
 * @param stored - What was stored for a view's features, parsed from wherever it is kept.
 * @returns What the view sets, held to the renderer's own bounds, with everything a view does not set dropped.
 */
export function toLevelFeatureOptions(stored: unknown): ILevelFeatureOptions {
  const overrides: IRenderFeatureOverrides = toRenderFeatureChoice({ overrides: stored }).overrides;

  function pick<K extends TLevelFeatureKey>(key: K): ILevelFeatureOptions[K] {
    const group: Record<string, unknown> = (overrides[key] ?? {}) as Record<string, unknown>;

    return Object.fromEntries(
      LEVEL_FEATURE_KEYS[key].filter((it) => group[it as string] !== undefined).map((it) => [it, group[it as string]])
    ) as ILevelFeatureOptions[K];
  }

  function pickModeFeature<K extends TLevelModeFeatureKey>(key: K): ILevelFeatureOptions[K] {
    const group: Record<string, unknown> = (overrides[key] ?? {}) as Record<string, unknown>;

    return Object.fromEntries(
      LEVEL_MODE_FEATURE_KEYS[key]
        .filter((it) => group[it as string] !== undefined)
        .map((it) => [it, group[it as string]])
    ) as ILevelFeatureOptions[K];
  }

  const antialiasing: RenderAntialiasing | undefined = overrides.antialiasing;

  return {
    ambientOcclusion: pick("ambientOcclusion"),
    antialiasing: antialiasing && LEVEL_ANTIALIASING_MODES.includes(antialiasing) ? antialiasing : null,
    debanding: pickModeFeature("debanding"),
    enhancedBloom: pickModeFeature("enhancedBloom"),
    fog: pickModeFeature("fog"),
    glass: pickModeFeature("glass"),
    grass: pick("grass"),
    indirectLight: pickModeFeature("indirectLight"),
    lights: pick("lights"),
    parallax: pickModeFeature("parallax"),
    rain: pickModeFeature("rain"),
    reflections: pickModeFeature("reflections"),
    shadows: pick("shadows"),
    textureFiltering: overrides.textureFiltering ?? null,
    upscaling: Object.fromEntries(
      (["scale", "sharpening", "dlssPreset", "isRayReconstructed"] as const)
        .filter((key) => overrides.upscaling?.[key] !== undefined && overrides.upscaling?.[key] !== null)
        .map((key) => [key, overrides.upscaling?.[key]])
    ),
    output: toLevelOutputOptions(stored),
    volumetric: pickModeFeature("volumetric"),
    water: pick("water"),
  };
}

/** The feature groups a view sets over the settings for itself, each behind a toolbar toggle of its own. */
export type TLevelFeatureKey = "ambientOcclusion" | "grass" | "lights" | "shadows" | "water";

/** A feature group as a view resolves it: what it draws with while on, and whether the settings let it be. */
export interface ILevelFeatureState<T> {
  value: T;
  isAvailable: boolean;
}

/**
 * @param key - A feature group.
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @param isOn - Whether the toolbar draws the group, which it can turn off but not on.
 * @returns The group the view is drawn with: the settings', the view's own values over them, a nested group's too.
 */
export function toLevelRendererFeature<K extends TLevelFeatureKey>(
  key: K,
  settings: IRenderFeatureSettings,
  view: ILevelFeatureOptions,
  isOn: boolean
): IRenderFeatureSettings[K] {
  const group: Record<string, unknown> = { ...settings[key] };

  for (const [name, value] of Object.entries(view[key] as Record<string, unknown>)) {
    const base: unknown = group[name];

    group[name] = isNestedGroup(base) && isNestedGroup(value) ? { ...base, ...value } : value;
  }

  return { ...group, isEnabled: settings[key].isEnabled && isOn } as IRenderFeatureSettings[K];
}

/**
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @returns The indirect light the view is drawn with: the settings', the view's own values over them.
 */
export function toLevelIndirectLight(
  settings: IRenderFeatureSettings,
  view: ILevelFeatureOptions
): TRenderIndirectLightSettings {
  return toLevelModeFeature("indirectLight", settings, view);
}

/**
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @returns The rain's wetting the view is drawn with: the settings', the view's own values over them.
 */
export function toLevelRain(settings: IRenderFeatureSettings, view: ILevelFeatureOptions): TRenderRainSettings {
  return toLevelModeFeature("rain", settings, view);
}

/**
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @returns The reflections the view is drawn with: the settings', the view's own values over them.
 */
export function toLevelReflections(
  settings: IRenderFeatureSettings,
  view: ILevelFeatureOptions
): TRenderReflectionSettings {
  return toLevelModeFeature("reflections", settings, view);
}

/**
 * @param key - A group no toolbar toggle turns off: the fog's, the sky debanding's, the bloom's, the indirect light's
 *   or the reflections'.
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @returns The group the view is drawn with: the settings', the view's own values over them.
 */
export function toLevelModeFeature<K extends TLevelModeFeatureKey>(
  key: K,
  settings: IRenderFeatureSettings,
  view: ILevelFeatureOptions
): IRenderFeatureSettings[K] {
  return { ...settings[key], ...view[key] };
}

/**
 * @param value - One value of a feature group.
 * @returns Whether it is a group of its own, as the contact shadows are within the shadows, rather than one value.
 */
function isNestedGroup(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * @param key - A feature group.
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @returns The group as the toolbar shows it: its values as they draw while on, and whether the settings allow it.
 */
export function toLevelFeatureState<K extends TLevelFeatureKey>(
  key: K,
  settings: IRenderFeatureSettings,
  view: ILevelFeatureOptions
): ILevelFeatureState<IRenderFeatureSettings[K]> {
  return { isAvailable: settings[key].isEnabled, value: toLevelRendererFeature(key, settings, view, true) };
}

/**
 * What a feature toggle's tooltip says: that the settings keep it off, what it draws with, or what the view is without.
 *
 * @param description - The feature's name, and what it says on and off.
 * @param description.label - Its name, as a sentence starts with it.
 * @param description.isPlural - Whether its name is, as "Shadows" is and "Grass" is not.
 * @param description.isAvailable - Whether the settings let it be on.
 * @param description.isOn - Whether the view has it on.
 * @param description.on - What it draws with, on.
 * @param description.off - What the view is without it.
 * @returns The line.
 */
export function describeLevelFeatureToggle(description: {
  label: string;
  isPlural?: boolean;
  isAvailable: boolean;
  isOn: boolean;
  on: string;
  off: string;
}): string {
  const { label, isPlural = false, isAvailable, isOn, on, off } = description;

  if (!isAvailable) {
    return `${label} ${isPlural ? "are" : "is"} off in Settings, under Rendering`;
  }

  return isOn ? on : off;
}

/**
 * @param features - The mode the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @param isAntialiased - Whether the toolbar smooths edges, which it can turn off but not on.
 * @returns The mode the view is drawn with.
 */
export function toLevelRendererAntialiasing(
  features: RenderAntialiasing,
  view: ILevelFeatureOptions,
  isAntialiased: boolean
): RenderAntialiasing {
  if (!isAntialiased || features === ERenderAntialiasing.NONE) {
    return ERenderAntialiasing.NONE;
  }

  return view.antialiasing ?? features;
}
