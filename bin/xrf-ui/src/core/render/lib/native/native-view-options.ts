import { Nullable } from "@xrf/types";

import {
  ERenderDebugView,
  ERenderSunShaftsQuality,
  ERenderSurfaceColor,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { TSettled } from "@/core/render/lib/settings/render-settled";

/** What a view decides of how its scene is shaded, beside the features every viewport shares. */
export type TNativeViewShading = TSettled<
  Pick<RenderViewOptions, "debugView" | "hemiStrength" | "isBumped" | "isLit" | "isWireframe" | "surfaceColor">
>;

/**
 * What a view switches of a scene: the weather, the grass, the water, the wall marks, the particles, the campfires and
 * the spawned objects' groups; and how its sunshafts step.
 */
export interface INativeViewSwitches extends Pick<
  RenderViewOptions,
  | "isClouded"
  | "isFogged"
  | "isLensFlared"
  | "isCampfireLit"
  | "isParticled"
  | "isRainy"
  | "isSkyHazed"
  | "isSkyVisible"
  | "isSpawnedItems"
  | "isSpawnedLamps"
  | "isSpawnedProps"
  | "isSpawnedReleased"
  | "isSpawnedWeapons"
  | "isSunShafted"
  | "isThundering"
  | "isWallmarked"
  | "isWindy"
  | "sunShafts"
> {
  /** Whether the grass the features plant is drawn in this view. */
  isGrassy: boolean;
  /** Whether the water the features draw is drawn in this view. */
  isWaterVisible: boolean;
}

/** How a scene is exposed, lit and corrected, beyond what its weather does. */
export type TNativeLook = TSettled<Pick<RenderViewOptions, "corrections" | "exposure" | "lightScales">>;

/** Nothing switched on: what an asset viewer's scene of one model has none of. */
export const NO_NATIVE_VIEW_SWITCHES: INativeViewSwitches = {
  isCampfireLit: true,
  isClouded: false,
  isFogged: false,
  isGrassy: false,
  isLensFlared: false,
  isParticled: false,
  isRainy: false,
  isSkyHazed: false,
  isSkyVisible: false,
  isSpawnedItems: true,
  isSpawnedLamps: true,
  isSpawnedProps: true,
  isSpawnedReleased: false,
  isSpawnedWeapons: true,
  isSunShafted: false,
  isThundering: false,
  isWallmarked: true,
  isWaterVisible: false,
  isWindy: false,
  sunShafts: { minimum: 0, quality: ERenderSunShaftsQuality.HIGH },
};

/** Nothing scaled and nothing corrected, as an asset viewer shows what it draws. */
export const NEUTRAL_NATIVE_LOOK: Omit<TNativeLook, "exposure"> = {
  corrections: { exposure: 1, gamma: 1, grading: [0, 0, 0], saturation: 1 },
  lightScales: { ambient: 1, hemi: 1, sun: 1 },
};

/**
 * @param resolution - How many pixels the settings ask a viewport be drawn with.
 * @returns How many rows a native viewport draws with at most; null for as many as it covers.
 */
export function toNativeRenderHeight(resolution: ERenderResolution): Nullable<number> {
  return resolution === ERenderResolution.WINDOW ? null : Number(resolution);
}

/**
 * @param shading - How the view shades its scene.
 * @param features - What the application sets every viewport's features to, as the view narrows them.
 * @param switches - What the view switches of its scene.
 * @param look - How the scene is exposed, lit and corrected.
 * @param renderHeight - How many rows the scene is drawn with at most; null for as many as the viewport covers.
 * @returns What a native viewport draws a level with; an asset viewer sets its own light and backdrop over it.
 */
export function toNativeViewOptions(
  shading: TNativeViewShading,
  features: IRenderFeatureSettings,
  switches: INativeViewSwitches,
  look: TNativeLook,
  renderHeight: Nullable<number> = null
): RenderViewOptions {
  const { grass, shadows, water } = features;
  const { corrections } = look;

  return {
    ...shading,
    ambientOcclusion: features.ambientOcclusion,
    antialiasing: features.antialiasing,
    assetLighting: null,
    backdrop: null,
    backdropSquares: null,
    checker: 0,
    corrections: { ...corrections, grading: [...corrections.grading] },
    exposure: look.exposure,
    grass: { ...grass, isEnabled: grass.isEnabled && switches.isGrassy },
    isAlphaVisible: true,
    isCampfireLit: switches.isCampfireLit,
    isClouded: switches.isClouded,
    isFogged: switches.isFogged,
    isLensFlared: switches.isLensFlared,
    isOcclusionCulled: features.isOcclusionCulled,
    isParticled: switches.isParticled,
    isRainy: switches.isRainy,
    isSkyHazed: switches.isSkyHazed,
    isSkyVisible: switches.isSkyVisible,
    isSpawnedItems: switches.isSpawnedItems,
    isSpawnedLamps: switches.isSpawnedLamps,
    isSpawnedProps: switches.isSpawnedProps,
    isSpawnedReleased: switches.isSpawnedReleased,
    isSpawnedWeapons: switches.isSpawnedWeapons,
    isSunShafted: switches.isSunShafted,
    isThundering: switches.isThundering,
    isWallmarked: switches.isWallmarked,
    isWindy: switches.isWindy,
    lightScales: look.lightScales,
    lights: features.lights,
    lod: features.lod,
    plainColor: null,
    renderHeight,
    shadows: { ...shadows, cascades: [...shadows.cascades] },
    sunShafts: { ...switches.sunShafts },
    upscaling: features.upscaling,
    water: { ...water, isEnabled: water.isEnabled && switches.isWaterVisible },
  };
}

/**
 * One asset against a backdrop, as the texture and visual viewers both draw it: textured, at the engine's noon scale
 * with the exposure held, since one asset is no scene to adapt it to, and with no sky or weather.
 *
 * @param shading - What the viewer's toolbar switches of the asset's shading.
 * @param features - What the application sets every viewport's features to.
 * @param renderHeight - How many rows the asset is drawn with at most; null for as many as the viewport covers.
 * @returns What a native viewport draws the asset with, before the viewer's light and backdrop.
 */
export function toNativeAssetViewOptions(
  shading: Pick<TNativeViewShading, "isBumped" | "isLit" | "isWireframe">,
  features: IRenderFeatureSettings,
  renderHeight: Nullable<number> = null
): RenderViewOptions {
  return toNativeViewOptions(
    {
      ...shading,
      debugView: ERenderDebugView.FINAL,
      hemiStrength: 1,
      surfaceColor: ERenderSurfaceColor.TEXTURED,
    },
    features,
    NO_NATIVE_VIEW_SWITCHES,
    { ...NEUTRAL_NATIVE_LOOK, exposure: { ...features.exposure, isEnabled: false } },
    renderHeight
  );
}
