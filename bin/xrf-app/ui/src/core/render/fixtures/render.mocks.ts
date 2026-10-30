import { ERenderAntialiasing, ERenderScale, RenderAppliedReport, RenderLoadReport } from "@xrf/ipc/types/xrf-renderer";

import { DEFAULT_RENDER_FEATURE_CHOICE, resolveRenderFeatures } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

/** What the tool sets every viewport's features to: the default preset's, unless told. */
export function mockRenderFeatures(overrides: Partial<IRenderFeatureSettings> = {}): IRenderFeatureSettings {
  return { ...resolveRenderFeatures(DEFAULT_RENDER_FEATURE_CHOICE), ...overrides };
}

/** How far the renderer has loaded a level: nothing of nothing and no part finished, unless told. */
export function mockRenderLoadReport(overrides: Partial<RenderLoadReport> = {}): RenderLoadReport {
  return {
    bytes: 0,
    durations: { grass: null, impostors: null, lights: null, particles: null, ready: null, sectors: null, spawn: null },
    impostors: 0,
    impostorsTotal: 0,
    isReady: false,
    scene: 0,
    sectors: 0,
    sectorsTotal: 0,
    textures: 0,
    texturesTotal: 0,
    ...overrides,
  };
}

/** What the renderer last applied to a viewport: TAA at the native scale and nothing else drawn, unless told. */
export function mockRenderAppliedReport(overrides: Partial<RenderAppliedReport> = {}): RenderAppliedReport {
  return {
    ambientOcclusion: null,
    ambientOcclusionMethod: null,
    antialiasing: ERenderAntialiasing.TAA,
    bloom: null,
    debanding: null,
    environment: null,
    grass: null,
    heightFog: null,
    indirectLight: null,
    isRayReconstructed: false,
    isWater: false,
    lights: null,
    reflections: null,
    renderScale: ERenderScale.NATIVE,
    shadows: null,
    sun: null,
    ...overrides,
  };
}
