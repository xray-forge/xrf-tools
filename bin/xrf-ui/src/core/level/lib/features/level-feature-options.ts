import { isRecord, Nullable } from "@xrf/types";

import { ERenderAntialiasing, RenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
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
  "bias" | "blend" | "cascades" | "contact" | "filter" | "resolution" | "swayCascades" | "swayRate"
>;

/** The ambient occlusion settings a level view may set for itself. */
export type TLevelAmbientOcclusionOptions = Pick<
  TRenderAmbientOcclusionSettings,
  "gtao" | "method" | "radius" | "rtao" | "strength" | "vbao"
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
  | "density"
  | "foliage"
  | "height"
  | "isAligned"
  | "isShadowed"
  | "radius"
  | "shadowCascades"
  | "shadowRadius"
  | "slopeLimit"
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

/** Every key of a group a view may set, as a table the compiler holds complete: a key left out is never stored. */
type TKeyTable<T> = { readonly [P in keyof Required<T>]: true };

/** The feature groups a view sets over the settings for itself, each behind a toolbar toggle of its own. */
export type TLevelFeatureKey = "ambientOcclusion" | "grass" | "lights" | "shadows" | "water";

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

/** Each group's settings a view may set, which is all a stored view keeps. */
const LEVEL_GROUP_KEYS: {
  readonly [K in TLevelFeatureKey | TLevelModeFeatureKey]: TKeyTable<ILevelFeatureOptions[K]>;
} = {
  ambientOcclusion: { gtao: true, method: true, radius: true, rtao: true, strength: true, vbao: true },
  grass: {
    density: true,
    foliage: true,
    height: true,
    isAligned: true,
    isShadowed: true,
    radius: true,
    shadowCascades: true,
    shadowRadius: true,
    slopeLimit: true,
  },
  lights: { isLevelLights: true, isShadowed: true, isVolumetric: true, shadowFilter: true },
  shadows: {
    bias: true,
    blend: true,
    cascades: true,
    contact: true,
    filter: true,
    resolution: true,
    swayCascades: true,
    swayRate: true,
  },
  water: {
    distortion: true,
    enhanced: true,
    isDistorted: true,
    isSoft: true,
    mode: true,
    reflection: true,
    ripple: true,
    waveHeight: true,
    waveSpeed: true,
  },
  debanding: { mode: true, passes: true, radius: true },
  enhancedBloom: { blur: true, exposure: true, mode: true, sky: true, threshold: true, vibrance: true },
  fog: { density: true, height: true, mode: true, scattering: true, skyFade: true, sunColor: true },
  glass: { mode: true, refraction: true },
  indirectLight: { intensity: true, mode: true, radius: true, steps: true },
  parallax: {
    blending: true,
    cavity: true,
    height: true,
    isRefined: true,
    isSteep: true,
    mode: true,
    range: true,
    shadow: true,
    steps: true,
    surfaceHeight: true,
    surfaceRange: true,
  },
  rain: {
    distance: true,
    dropSquare: true,
    drops: true,
    isReliefFlooded: true,
    mode: true,
    puddles: true,
    reflectivity: true,
    ringDistance: true,
    ringSize: true,
    ringStrength: true,
    ripples: true,
  },
  reflections: {
    distance: true,
    intensity: true,
    isDryTraced: true,
    isGrassReflected: true,
    isRayBudgeted: true,
    isTraced: true,
    isWaterReflected: true,
    mode: true,
    probe: true,
    reach: true,
    skyGain: true,
    steps: true,
    traceInterval: true,
    traceRoughness: true,
    traceSize: true,
    waterSteps: true,
  },
  volumetric: { mode: true, intensity: true, anisotropy: true, distance: true },
};

/**
 * @param stored - What was stored for a view's features, parsed from wherever it is kept.
 * @returns What the view sets, held to the renderer's own bounds, with everything a view does not set dropped.
 */
export function toLevelFeatureOptions(stored: unknown): ILevelFeatureOptions {
  const overrides: IRenderFeatureOverrides = toRenderFeatureChoice({ overrides: stored }).overrides;

  function pick<K extends TLevelFeatureKey | TLevelModeFeatureKey>(key: K): ILevelFeatureOptions[K] {
    const group: Record<string, unknown> = (overrides[key] ?? {}) as Record<string, unknown>;

    return Object.fromEntries(
      Object.keys(LEVEL_GROUP_KEYS[key])
        .filter((it: string) => group[it] !== undefined)
        .map((it: string) => [it, group[it]])
    ) as ILevelFeatureOptions[K];
  }

  const antialiasing: RenderAntialiasing | undefined = overrides.antialiasing;

  return {
    ambientOcclusion: pick("ambientOcclusion"),
    antialiasing: antialiasing && LEVEL_ANTIALIASING_MODES.includes(antialiasing) ? antialiasing : null,
    debanding: pick("debanding"),
    enhancedBloom: pick("enhancedBloom"),
    fog: pick("fog"),
    glass: pick("glass"),
    grass: pick("grass"),
    indirectLight: pick("indirectLight"),
    lights: pick("lights"),
    parallax: pick("parallax"),
    rain: pick("rain"),
    reflections: pick("reflections"),
    shadows: pick("shadows"),
    upscaling: Object.fromEntries(
      (["scale", "sharpening", "dlssPreset", "isRayReconstructed", "rayReconstructionPreset"] as const)
        .filter((key) => overrides.upscaling?.[key] !== undefined && overrides.upscaling?.[key] !== null)
        .map((key) => [key, overrides.upscaling?.[key]])
    ),
    output: toLevelOutputOptions(stored),
    volumetric: pick("volumetric"),
    water: pick("water"),
  };
}

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

    group[name] = isRecord(base) && isRecord(value) ? { ...base, ...value } : value;
  }

  return { ...group, isEnabled: settings[key].isEnabled && isOn } as IRenderFeatureSettings[K];
}

/**
 * @param key - A group no toolbar toggle turns off, one of `TLevelModeFeatureKey`.
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

/** The toolbar's toggles that turn a feature group off for the view, which they cannot turn on. */
export type TLevelFeatureToggles = Pick<
  ILevelViewOptions,
  | "isAntialiased"
  | "isDepthPrepassed"
  | "isGrassy"
  | "isLamplit"
  | "isOccluded"
  | "isOcclusionCulled"
  | "isShadowed"
  | "isWaterVisible"
>;

/** Every toggle on: the features as the settings and the view set them, none turned off. */
export const LEVEL_FEATURE_TOGGLES_ON: Readonly<TLevelFeatureToggles> = {
  isAntialiased: true,
  isDepthPrepassed: true,
  isGrassy: true,
  isLamplit: true,
  isOccluded: true,
  isOcclusionCulled: true,
  isShadowed: true,
  isWaterVisible: true,
};

/**
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @param toggles - What the toolbar turns off.
 * @returns Every feature the view is drawn with: the settings', the view's own over them, under the toggles; the level
 *   of detail as the settings set it.
 */
export function toLevelRendererFeatures(
  settings: IRenderFeatureSettings,
  view: ILevelFeatureOptions,
  toggles: TLevelFeatureToggles
): IRenderFeatureSettings {
  return {
    ...settings,
    ambientOcclusion: toLevelRendererFeature("ambientOcclusion", settings, view, toggles.isOccluded),
    antialiasing: toLevelRendererAntialiasing(settings.antialiasing, view, toggles.isAntialiased),
    debanding: toLevelModeFeature("debanding", settings, view),
    enhancedBloom: toLevelModeFeature("enhancedBloom", settings, view),
    fog: toLevelModeFeature("fog", settings, view),
    glass: toLevelModeFeature("glass", settings, view),
    grass: toLevelRendererFeature("grass", settings, view, toggles.isGrassy),
    indirectLight: toLevelModeFeature("indirectLight", settings, view),
    isDepthPrepassed: settings.isDepthPrepassed && toggles.isDepthPrepassed,
    isOcclusionCulled: settings.isOcclusionCulled && toggles.isOcclusionCulled,
    lights: toLevelRendererFeature("lights", settings, view, toggles.isLamplit),
    parallax: toLevelModeFeature("parallax", settings, view),
    rain: toLevelModeFeature("rain", settings, view),
    reflections: toLevelModeFeature("reflections", settings, view),
    shadows: toLevelRendererFeature("shadows", settings, view, toggles.isShadowed),
    upscaling: { ...settings.upscaling, ...view.upscaling },
    volumetric: toLevelModeFeature("volumetric", settings, view),
    water: toLevelRendererFeature("water", settings, view, toggles.isWaterVisible),
  };
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
