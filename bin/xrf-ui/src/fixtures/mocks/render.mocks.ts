import { DEFAULT_RENDER_FEATURE_CHOICE, resolveRenderFeatures } from "@/core/render/lib/settings/render-feature-choice";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

/** What the application sets every viewport's features to: the default preset's, unless told. */
export function mockRenderFeatures(overrides: Partial<IRenderFeatureSettings> = {}): IRenderFeatureSettings {
  return { ...resolveRenderFeatures(DEFAULT_RENDER_FEATURE_CHOICE), ...overrides };
}
