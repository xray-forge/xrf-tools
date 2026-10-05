import { RenderLoadReport } from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_RENDER_FEATURE_CHOICE, resolveRenderFeatures } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

/** What the application sets every viewport's features to: the default preset's, unless told. */
export function mockRenderFeatures(overrides: Partial<IRenderFeatureSettings> = {}): IRenderFeatureSettings {
  return { ...resolveRenderFeatures(DEFAULT_RENDER_FEATURE_CHOICE), ...overrides };
}

/** How far the renderer has loaded a level: nothing of nothing and no part finished, unless told. */
export function mockRenderLoadReport(overrides: Partial<RenderLoadReport> = {}): RenderLoadReport {
  return {
    bytes: 0,
    durations: { grass: null, lights: null, particles: null, ready: null, sectors: null, spawn: null },
    isReady: false,
    sectors: 0,
    sectorsTotal: 0,
    textures: 0,
    texturesTotal: 0,
    ...overrides,
  };
}
