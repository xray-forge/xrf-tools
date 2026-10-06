import { Nullable } from "@xrf/types";

import {
  ERenderDebugView,
  ERenderSunShaftsQuality,
  ERenderSurfaceColor,
  RenderShowFlags,
  RenderViewFeatures,
  RenderViewMode,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import { WorldToggles } from "@/core/ipc/types/xrf-world";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { TSettled } from "@/core/render/lib/settings/render-settled";

/** What a view decides of how its scene is shaded, beside the features every viewport shares. */
export type TNativeViewShading = TSettled<
  Pick<RenderViewMode, "debugView" | "isBumped" | "isLit" | "isWireframe" | "surfaceColor"> &
    Pick<RenderViewFeatures, "hemiStrength">
>;

/**
 * What a view switches of how a scene is drawn: the sky and its weather, the grass, the water, the wall marks and the
 * particles; and how its sunshafts step. What of the world plays is the world's, {@link toNativeWorldToggles}.
 */
export interface INativeViewSwitches
  extends
    Pick<
      RenderShowFlags,
      | "isClouded"
      | "isFogged"
      | "isLensFlared"
      | "isParticled"
      | "isSkyHazed"
      | "isSkyVisible"
      | "isSunShafted"
      | "isWallmarked"
    >,
    Pick<RenderViewFeatures, "sunShafts"> {
  /** Whether the grass the features plant is drawn in this view. */
  isGrassy: boolean;
  /** Whether the water the features draw is drawn in this view. */
  isWaterVisible: boolean;
}

/** How a scene is exposed, lit, bloomed and corrected, beyond what its weather does. */
export type TNativeLook = TSettled<Pick<RenderViewFeatures, "bloom" | "corrections" | "exposure" | "lightScales">>;

/** Nothing switched on: what an asset viewer's scene of one model has none of. */
export const NO_NATIVE_VIEW_SWITCHES: INativeViewSwitches = {
  isClouded: false,
  isFogged: false,
  isGrassy: false,
  isLensFlared: false,
  isParticled: false,
  isSkyHazed: false,
  isSkyVisible: false,
  isSunShafted: false,
  isWallmarked: true,
  isWaterVisible: false,
  sunShafts: { minimum: 0, quality: ERenderSunShaftsQuality.HIGH },
};

/** Nothing scaled, bloomed or corrected, as an asset viewer shows what it draws. */
export const NEUTRAL_NATIVE_LOOK: Omit<TNativeLook, "exposure"> = {
  bloom: { isEnabled: false, radius: 3, strength: 0.7, threshold: 0.00001 },
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
 * @returns What a native viewport draws a level with; an asset viewer sets its own light and backdrop over it. What of
 *   the world plays goes to the world apart, {@link toNativeWorldToggles}.
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
    asset: { backdrop: null, backdropSquares: null, checker: 0, lighting: null, plainColor: null },
    features: {
      ambientOcclusion: features.ambientOcclusion,
      antialiasing: features.antialiasing,
      bloom: look.bloom,
      corrections: { ...corrections, grading: [...corrections.grading] },
      exposure: look.exposure,
      grass: { ...grass, isEnabled: grass.isEnabled && switches.isGrassy },
      hemiStrength: shading.hemiStrength,
      indirectLight: features.indirectLight,
      isOcclusionCulled: features.isOcclusionCulled,
      lightScales: look.lightScales,
      lights: features.lights,
      rain: features.rain,
      reflections: features.reflections,
      lod: features.lod,
      shadows: { ...shadows, cascades: [...shadows.cascades] },
      sunShafts: { ...switches.sunShafts },
      water: { ...water, isEnabled: water.isEnabled && switches.isWaterVisible },
    },
    mode: {
      debugView: shading.debugView,
      isBumped: shading.isBumped,
      isLit: shading.isLit,
      isWireframe: shading.isWireframe,
      surfaceColor: shading.surfaceColor,
    },
    output: { renderHeight, upscaling: features.upscaling },
    show: {
      isAlphaVisible: true,
      isClouded: switches.isClouded,
      isFogged: switches.isFogged,
      isLensFlared: switches.isLensFlared,
      isParticled: switches.isParticled,
      isSkyHazed: switches.isSkyHazed,
      isSkyVisible: switches.isSkyVisible,
      isSunShafted: switches.isSunShafted,
      isWallmarked: switches.isWallmarked,
    },
  };
}

/**
 * @param switches - What the view switches of its scene, the world's part among them.
 * @returns What of the level's world plays in the view: the weather's rain, bolts and wind, the campfires, the ambient
 *   effects and the spawned objects' groups.
 */
export function toNativeWorldToggles(switches: WorldToggles): WorldToggles {
  return {
    isAmbientPlayed: switches.isAmbientPlayed,
    isCampfireLit: switches.isCampfireLit,
    isRainy: switches.isRainy,
    isSpawnedItems: switches.isSpawnedItems,
    isSpawnedLamps: switches.isSpawnedLamps,
    isSpawnedProps: switches.isSpawnedProps,
    isSpawnedReleased: switches.isSpawnedReleased,
    isSpawnedWeapons: switches.isSpawnedWeapons,
    isThundering: switches.isThundering,
    isWindy: switches.isWindy,
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
