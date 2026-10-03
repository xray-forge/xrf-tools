import {
  ERendererAmbientOcclusionQuality,
  ERendererAntialiasing,
  ERendererDebugView,
  ERendererLightShadowFilter,
  ERendererRenderScale,
  ERenderResolution,
  IRendererSettings,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import {
  ERenderAmbientOcclusionQuality,
  ERenderAntialiasing,
  ERenderDebugView,
  ERenderLightShadowFilter,
  ERenderScale,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";

/** The settings' occlusion qualities as the native renderer names them. */
const AMBIENT_OCCLUSION_QUALITIES: Record<ERendererAmbientOcclusionQuality, ERenderAmbientOcclusionQuality> = {
  [ERendererAmbientOcclusionQuality.LOW]: ERenderAmbientOcclusionQuality.LOW,
  [ERendererAmbientOcclusionQuality.MEDIUM]: ERenderAmbientOcclusionQuality.MEDIUM,
  [ERendererAmbientOcclusionQuality.HIGH]: ERenderAmbientOcclusionQuality.HIGH,
  [ERendererAmbientOcclusionQuality.ULTRA]: ERenderAmbientOcclusionQuality.ULTRA,
};

/** The debug views as the native renderer names them. */
const DEBUG_VIEWS: Record<ERendererDebugView, ERenderDebugView> = {
  [ERendererDebugView.FINAL]: ERenderDebugView.FINAL,
  [ERendererDebugView.ALBEDO]: ERenderDebugView.ALBEDO,
  [ERendererDebugView.GLOSS]: ERenderDebugView.GLOSS,
  [ERendererDebugView.NORMAL]: ERenderDebugView.NORMAL,
  [ERendererDebugView.HEMI]: ERenderDebugView.HEMI,
  [ERendererDebugView.SUN]: ERenderDebugView.SUN,
  [ERendererDebugView.MATERIAL]: ERenderDebugView.MATERIAL,
  [ERendererDebugView.DEPTH]: ERenderDebugView.DEPTH,
  [ERendererDebugView.LIGHT]: ERenderDebugView.LIGHT,
  [ERendererDebugView.AMBIENT_OCCLUSION]: ERenderDebugView.AMBIENT_OCCLUSION,
  [ERendererDebugView.MOTION]: ERenderDebugView.MOTION,
};

/** The settings' antialiasing as the native renderer names it. */
const ANTIALIASING_MODES: Record<ERendererAntialiasing, ERenderAntialiasing> = {
  [ERendererAntialiasing.NONE]: ERenderAntialiasing.NONE,
  [ERendererAntialiasing.FXAA]: ERenderAntialiasing.FXAA,
  [ERendererAntialiasing.SMAA]: ERenderAntialiasing.SMAA,
  [ERendererAntialiasing.TAA]: ERenderAntialiasing.TAA,
  [ERendererAntialiasing.FSR2]: ERenderAntialiasing.FSR2,
};

/** The settings' render scales as the native renderer names them. */
const RENDER_SCALES: Record<ERendererRenderScale, ERenderScale> = {
  [ERendererRenderScale.NATIVE]: ERenderScale.NATIVE,
  [ERendererRenderScale.QUALITY]: ERenderScale.QUALITY,
  [ERendererRenderScale.BALANCED]: ERenderScale.BALANCED,
  [ERendererRenderScale.PERFORMANCE]: ERenderScale.PERFORMANCE,
};

/** The settings' light shadow filters as the native renderer names them. */
const LIGHT_SHADOW_FILTERS: Record<ERendererLightShadowFilter, ERenderLightShadowFilter> = {
  [ERendererLightShadowFilter.ENGINE]: ERenderLightShadowFilter.ENGINE,
  [ERendererLightShadowFilter.SOFT]: ERenderLightShadowFilter.SOFT,
};

/** Nothing switched on: what an asset viewer's scene of one model has none of. */
export const NO_NATIVE_VIEW_SWITCHES: INativeViewSwitches = {
  isClouded: false,
  isFogged: false,
  isGrassy: false,
  isRainy: false,
  isSkyHazed: false,
  isSkyVisible: false,
  isSpawnedItems: true,
  isSpawnedLamps: true,
  isSpawnedProps: true,
  isSpawnedWeapons: true,
  isThundering: false,
  isWallmarked: true,
  isWaterVisible: false,
  isWindy: false,
};

/**
 * @param resolution - How many pixels the settings ask a viewport be drawn with.
 * @returns How many rows a native viewport draws with at most; null for as many as it covers.
 */
export function toNativeRenderHeight(resolution: ERenderResolution): Nullable<number> {
  return resolution === ERenderResolution.WINDOW ? null : Number(resolution);
}

/** What a view switches of a scene: the weather, the grass, the water, the wall marks and the spawned objects' groups. */
export interface INativeViewSwitches {
  isClouded: boolean;
  isFogged: boolean;
  isGrassy: boolean;
  isRainy: boolean;
  isSkyHazed: boolean;
  isSkyVisible: boolean;
  isSpawnedItems: boolean;
  isSpawnedLamps: boolean;
  isSpawnedProps: boolean;
  isSpawnedWeapons: boolean;
  isThundering: boolean;
  isWallmarked: boolean;
  isWaterVisible: boolean;
  isWindy: boolean;
}

/** How a scene is exposed, lit and corrected, beyond what its weather does. */
export type TNativeLook = Pick<RenderViewOptions, "corrections" | "exposure" | "lightScales">;

/** Nothing scaled and nothing corrected, as an asset viewer shows what it draws. */
export const NEUTRAL_NATIVE_LOOK: Omit<TNativeLook, "exposure"> = {
  corrections: { exposure: 1, gamma: 1, grading: [0, 0, 0], saturation: 1 },
  lightScales: { ambient: 1, hemi: 1, sun: 1 },
};

/**
 * @param settings - What a view's toolbar and the application's settings come to.
 * @param switches - What the view switches of its scene.
 * @param look - How the scene is exposed, lit and corrected.
 * @param renderHeight - How many rows the scene is drawn with at most; null for as many as the viewport covers.
 * @returns What a native viewport draws a level with; an asset viewer sets its own light and backdrop over it.
 */
export function toNativeViewOptions(
  settings: IRendererSettings,
  switches: INativeViewSwitches,
  look: TNativeLook,
  renderHeight: Nullable<number> = null
): RenderViewOptions {
  const { ambientOcclusion, grass, lights, lod, shadows, water } = settings.features;
  const { corrections, exposure, lightScales } = look;

  return {
    ambientOcclusion: {
      isEnabled: ambientOcclusion.isEnabled,
      quality: AMBIENT_OCCLUSION_QUALITIES[ambientOcclusion.quality],
      radius: ambientOcclusion.radius,
      strength: ambientOcclusion.strength,
    },
    antialiasing: ANTIALIASING_MODES[settings.features.antialiasing],
    assetLighting: null,
    backdrop: null,
    checker: 0,
    corrections: {
      exposure: corrections.exposure,
      gamma: corrections.gamma,
      grading: corrections.grading,
      saturation: corrections.saturation,
    },
    debugView: DEBUG_VIEWS[settings.debugView],
    exposure: {
      adaptation: exposure.adaptation,
      amount: exposure.amount,
      isEnabled: exposure.isEnabled,
      lowLuminance: exposure.lowLuminance,
      middleGray: exposure.middleGray,
    },
    grass: {
      density: grass.density,
      height: grass.height,
      isEnabled: grass.isEnabled && switches.isGrassy,
      radius: grass.radius,
    },
    hemiStrength: settings.hemiStrength,
    isAlphaVisible: true,
    isBumped: settings.isBumped,
    isClouded: switches.isClouded,
    isFogged: switches.isFogged,
    isLit: settings.isLit,
    isOcclusionCulled: settings.features.isOcclusionCulled,
    isRainy: switches.isRainy,
    isSkyHazed: switches.isSkyHazed,
    isSkyVisible: switches.isSkyVisible,
    isSpawnedItems: switches.isSpawnedItems,
    isSpawnedLamps: switches.isSpawnedLamps,
    isSpawnedProps: switches.isSpawnedProps,
    isSpawnedWeapons: switches.isSpawnedWeapons,
    isTextured: settings.isTextured,
    isThundering: switches.isThundering,
    isWallmarked: switches.isWallmarked,
    isWindy: switches.isWindy,
    isWireframe: settings.isWireframe,
    lightScales: { ambient: lightScales.ambient, hemi: lightScales.hemi, sun: lightScales.sun },
    lights: {
      isEnabled: lights.isEnabled,
      isLevelLights: lights.isLevelLights,
      isShadowed: lights.isShadowed,
      shadowFilter: LIGHT_SHADOW_FILTERS[lights.shadowFilter],
    },
    lod: {
      geometryLod: lod.geometryLod,
      isImpostors: lod.isImpostors,
      ssaA: lod.ssaA,
      ssaB: lod.ssaB,
      ssaDiscard: lod.ssaDiscard,
      ssaGlodEnd: lod.ssaGlodEnd,
      ssaGlodStart: lod.ssaGlodStart,
    },
    renderHeight,
    shadows: {
      bias: shadows.bias,
      blend: shadows.blend,
      cascades: [...shadows.cascades],
      filter: shadows.filter,
      isEnabled: shadows.isEnabled,
      isStaggered: shadows.isStaggered,
      reach: shadows.reach,
      resolution: shadows.resolution,
    },
    tonemapScale: settings.tonemapScale,
    upscaling: {
      scale: RENDER_SCALES[settings.features.upscaling.scale],
      sharpening: settings.features.upscaling.sharpening,
    },
    water: {
      distortion: water.distortion,
      isDistorted: water.isDistorted,
      isEnabled: water.isEnabled && switches.isWaterVisible,
      isSoft: water.isSoft,
      reflection: water.reflection,
      ripple: water.ripple,
      waveHeight: water.waveHeight,
      waveSpeed: water.waveSpeed,
    },
  };
}
