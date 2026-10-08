import { Nullable } from "@xrf/types";

import { ERenderAntialiasing, RenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import { toRenderFeatureChoice } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import {
  IRenderFeatureSettings,
  TRenderAmbientOcclusionSettings,
  TRenderGrassSettings,
  TRenderIndirectLightSettings,
  TRenderLightsSettings,
  TRenderReflectionSettings,
  TRenderShadowSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/render-feature-settings";

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

/** The grass settings a level view may set for itself. */
export type TLevelGrassOptions = Pick<TRenderGrassSettings, "density" | "foliage" | "height" | "radius">;

/** The water settings a level view may set for itself: all but whether it is drawn, which the toolbar toggles. */
export type TLevelWaterOptions = Omit<TRenderWaterSettings, "isEnabled">;

/** The lights settings a level view may set for itself. */
export type TLevelLightsOptions = Pick<TRenderLightsSettings, "isLevelLights" | "isShadowed" | "shadowFilter">;

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
  /** The mode edges are smoothed with while the settings smooth them at all, or null for the settings' own. */
  antialiasing: Nullable<RenderAntialiasing>;
  shadows: Partial<TLevelShadowOptions>;
  water: Partial<TLevelWaterOptions>;
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
  grass: ["density", "foliage", "height", "radius"],
  lights: ["isLevelLights", "isShadowed", "shadowFilter"],
  shadows: ["bias", "blend", "cascades", "contact", "filter", "resolution"],
  water: ["distortion", "enhanced", "isDistorted", "isSoft", "mode", "reflection", "ripple", "waveHeight", "waveSpeed"],
};

/** The indirect light's settings a view may set. */
const LEVEL_INDIRECT_LIGHT_KEYS: ReadonlyArray<keyof TLevelIndirectLightOptions> = ["intensity", "mode", "radius"];

/** The reflections' settings a view may set. */
const LEVEL_REFLECTION_KEYS: ReadonlyArray<keyof TLevelReflectionOptions> = [
  "distance",
  "intensity",
  "mode",
  "quality",
];

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

  const antialiasing: RenderAntialiasing | undefined = overrides.antialiasing;
  const indirectLight: Record<string, unknown> = (overrides.indirectLight ?? {}) as Record<string, unknown>;
  const reflections: Record<string, unknown> = (overrides.reflections ?? {}) as Record<string, unknown>;

  return {
    ambientOcclusion: pick("ambientOcclusion"),
    antialiasing: antialiasing && LEVEL_ANTIALIASING_MODES.includes(antialiasing) ? antialiasing : null,
    grass: pick("grass"),
    indirectLight: Object.fromEntries(
      LEVEL_INDIRECT_LIGHT_KEYS.filter((it) => indirectLight[it] !== undefined).map((it) => [it, indirectLight[it]])
    ) as Partial<TLevelIndirectLightOptions>,
    lights: pick("lights"),
    reflections: Object.fromEntries(
      LEVEL_REFLECTION_KEYS.filter((it) => reflections[it] !== undefined).map((it) => [it, reflections[it]])
    ) as Partial<TLevelReflectionOptions>,
    shadows: pick("shadows"),
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

/** Every feature group a view sets, resolved as its toolbar shows them. */
export type TLevelFeatureView = { readonly [K in TLevelFeatureKey]: ILevelFeatureState<IRenderFeatureSettings[K]> };

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
  return { ...settings.indirectLight, ...view.indirectLight };
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
  return { ...settings.reflections, ...view.reflections };
}

/**
 * @param value - One value of a feature group.
 * @returns Whether it is a group of its own, as the contact shadows are within the shadows, rather than one value.
 */
function isNestedGroup(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/**
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @returns Each group as the toolbar shows it: its values as they draw while on, and whether the settings allow it.
 */
export function toLevelFeatureView(settings: IRenderFeatureSettings, view: ILevelFeatureOptions): TLevelFeatureView {
  function toState<K extends TLevelFeatureKey>(key: K): ILevelFeatureState<IRenderFeatureSettings[K]> {
    return { isAvailable: settings[key].isEnabled, value: toLevelRendererFeature(key, settings, view, true) };
  }

  return {
    ambientOcclusion: toState("ambientOcclusion"),
    grass: toState("grass"),
    lights: toState("lights"),
    shadows: toState("shadows"),
    water: toState("water"),
  };
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
