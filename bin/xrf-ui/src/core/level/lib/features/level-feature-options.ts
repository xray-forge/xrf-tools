import { Nullable } from "@xrf/types";

import { ERenderAntialiasing, RenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import { toRenderFeatureChoice } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureOverrides } from "@/core/render/lib/settings/render-feature-overrides";
import {
  IRenderFeatureSettings,
  TRenderAmbientOcclusionSettings,
  TRenderGrassSettings,
  TRenderLightsSettings,
  TRenderShadowSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/render-feature-settings";

/** The shadow settings a level view may set for itself. */
export type TLevelShadowOptions = Pick<TRenderShadowSettings, "bias" | "blend" | "cascades" | "filter" | "resolution">;

/** The ambient occlusion settings a level view may set for itself. */
export type TLevelAmbientOcclusionOptions = Pick<TRenderAmbientOcclusionSettings, "quality" | "radius" | "strength">;

/** The grass settings a level view may set for itself. */
export type TLevelGrassOptions = Pick<TRenderGrassSettings, "density" | "height" | "radius">;

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
  lights: Partial<TLevelLightsOptions>;
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
  ambientOcclusion: ["quality", "radius", "strength"],
  grass: ["density", "height", "radius"],
  lights: ["isLevelLights", "isShadowed", "shadowFilter"],
  shadows: ["bias", "blend", "cascades", "filter", "resolution"],
  water: [
    "blurNoise",
    "caustics",
    "distortion",
    "isDistorted",
    "isSoft",
    "mode",
    "reflection",
    "reflectionBlur",
    "reflectivity",
    "refraction",
    "ripple",
    "softBorder",
    "specular",
    "turbidity",
    "waveHeight",
    "waveSpeed",
  ],
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

  const antialiasing: RenderAntialiasing | undefined = overrides.antialiasing;

  return {
    ambientOcclusion: pick("ambientOcclusion"),
    antialiasing: antialiasing && LEVEL_ANTIALIASING_MODES.includes(antialiasing) ? antialiasing : null,
    grass: pick("grass"),
    lights: pick("lights"),
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
 * @returns The group the view is drawn with: the settings', the view's own values over them.
 */
export function toLevelRendererFeature<K extends TLevelFeatureKey>(
  key: K,
  settings: IRenderFeatureSettings,
  view: ILevelFeatureOptions,
  isOn: boolean
): IRenderFeatureSettings[K] {
  return { ...settings[key], ...view[key], isEnabled: settings[key].isEnabled && isOn } as IRenderFeatureSettings[K];
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
