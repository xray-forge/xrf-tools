import {
  ERendererAntialiasing,
  IRendererAmbientOcclusionSettings,
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
export const LEVEL_ANTIALIASING_MODES: ReadonlyArray<ERendererAntialiasing> = [
  ERendererAntialiasing.FXAA,
  ERendererAntialiasing.SMAA,
  ERendererAntialiasing.TAA,
  ERendererAntialiasing.FSR,
];

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

/**
 * @param features - The ambient occlusion the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @param isOccluded - Whether the toolbar draws it, which it can turn off but not on.
 * @returns The ambient occlusion the view is drawn with.
 */
export function toLevelRendererAmbientOcclusion(
  features: IRendererAmbientOcclusionSettings,
  view: ILevelFeatureOptions,
  isOccluded: boolean
): IRendererAmbientOcclusionSettings {
  return { ...features, ...view.ambientOcclusion, isEnabled: features.isEnabled && isOccluded };
}

/**
 * @param features - The grass the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @param isGrassy - Whether the toolbar draws it, which it can turn off but not on.
 * @returns The grass the view is drawn with.
 */
export function toLevelRendererGrassSettings(
  features: IRendererGrassSettings,
  view: ILevelFeatureOptions,
  isGrassy: boolean
): IRendererGrassSettings {
  return { ...features, ...view.grass, isEnabled: features.isEnabled && isGrassy };
}

/**
 * @param features - The lights the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @param isLightsOn - Whether the toolbar draws them, which it can turn off but not on.
 * @returns The lights the view is drawn with.
 */
export function toLevelRendererLightsSettings(
  features: IRendererLightsSettings,
  view: ILevelFeatureOptions,
  isLightsOn: boolean
): IRendererLightsSettings {
  return { ...features, ...view.lights, isEnabled: features.isEnabled && isLightsOn };
}

/**
 * @param features - The shadows the renderer's settings set, for every viewport.
 * @param view - What the view sets over them.
 * @param isShadowed - Whether the toolbar draws shadows, which it can turn off but not on.
 * @returns The shadows the view is drawn with.
 */
export function toLevelRendererShadows(
  features: IRendererShadowSettings,
  view: ILevelFeatureOptions,
  isShadowed: boolean
): IRendererShadowSettings {
  return { ...features, ...view.shadows, isEnabled: features.isEnabled && isShadowed };
}
