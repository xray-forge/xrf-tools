import {
  ERendererAntialiasing,
  IRendererAmbientOcclusionSettings,
  IRendererFeatureSettings,
  IRendererGrassSettings,
  IRendererLightsSettings,
  IRendererShadowSettings,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

/** The shadow settings a level view may set for itself. */
export type TLevelShadowOptions = Pick<
  IRendererShadowSettings,
  "bias" | "blend" | "cascades" | "filter" | "resolution"
>;

/** The ambient occlusion settings a level view may set for itself. */
export type TLevelAmbientOcclusionOptions = Pick<IRendererAmbientOcclusionSettings, "quality" | "radius" | "strength">;

/** The grass settings a level view may set for itself. */
export type TLevelGrassOptions = Pick<IRendererGrassSettings, "density" | "height" | "radius">;

/** The lights settings a level view may set for itself. */
export type TLevelLightsOptions = Pick<IRendererLightsSettings, "isLevelLights" | "isShadowed" | "shadowFilter">;

/**
 * What a level view sets over the renderer's settings for itself: whatever it leaves unset follows them.
 */
export interface ILevelFeatureOptions {
  ambientOcclusion: Partial<TLevelAmbientOcclusionOptions>;
  grass: Partial<TLevelGrassOptions>;
  lights: Partial<TLevelLightsOptions>;
  /** The mode edges are smoothed with while the settings smooth them at all, or null for the settings' own. */
  antialiasing: Nullable<ERendererAntialiasing>;
  shadows: Partial<TLevelShadowOptions>;
}

export const DEFAULT_LEVEL_FEATURE_OPTIONS: ILevelFeatureOptions = {
  ambientOcclusion: {},
  antialiasing: null,
  grass: {},
  lights: {},
  shadows: {},
};

/** The modes a view picks from, which smooth something: turning it off is the toggle's. */
export const LEVEL_ANTIALIASING_MODES: ReadonlyArray<ERendererAntialiasing> = Object.values(
  ERendererAntialiasing
).filter((mode: ERendererAntialiasing) => mode !== ERendererAntialiasing.NONE);

/** The feature groups a view sets over the settings for itself, each behind a toolbar toggle of its own. */
export type TLevelFeatureKey = "ambientOcclusion" | "grass" | "lights" | "shadows";

/** A feature group as a view resolves it: what it draws with while on, and whether the settings let it be. */
export interface ILevelFeatureState<T> {
  value: T;
  isAvailable: boolean;
}

/** Every feature group a view sets, resolved as its toolbar shows them. */
export type TLevelFeatureView = { readonly [K in TLevelFeatureKey]: ILevelFeatureState<IRendererFeatureSettings[K]> };

/**
 * @param key - A feature group.
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @param isOn - Whether the toolbar draws the group, which it can turn off but not on.
 * @returns The group the view is drawn with: the settings', the view's own values over them.
 */
export function toLevelRendererFeature<K extends TLevelFeatureKey>(
  key: K,
  settings: IRendererFeatureSettings,
  view: ILevelFeatureOptions,
  isOn: boolean
): IRendererFeatureSettings[K] {
  return { ...settings[key], ...view[key], isEnabled: settings[key].isEnabled && isOn } as IRendererFeatureSettings[K];
}

/**
 * @param settings - What the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @returns Each group as the toolbar shows it: its values as they draw while on, and whether the settings allow it.
 */
export function toLevelFeatureView(settings: IRendererFeatureSettings, view: ILevelFeatureOptions): TLevelFeatureView {
  function toState<K extends TLevelFeatureKey>(key: K): ILevelFeatureState<IRendererFeatureSettings[K]> {
    return { isAvailable: settings[key].isEnabled, value: toLevelRendererFeature(key, settings, view, true) };
  }

  return {
    ambientOcclusion: toState("ambientOcclusion"),
    grass: toState("grass"),
    lights: toState("lights"),
    shadows: toState("shadows"),
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
  features: ERendererAntialiasing,
  view: ILevelFeatureOptions,
  isAntialiased: boolean
): ERendererAntialiasing {
  if (!isAntialiased || features === ERendererAntialiasing.NONE) {
    return ERendererAntialiasing.NONE;
  }

  return view.antialiasing ?? features;
}
