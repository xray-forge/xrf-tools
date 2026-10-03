import { DEFAULT_RENDER_FRAME_PACING } from "@/core/render/lib/contract/render-frame-pacing";
import {
  DEFAULT_RENDERER_FEATURE_CHOICE,
  resolveRendererFeatures,
} from "@/core/render/lib/contract/renderer-feature-choice";
import { IRenderSharedSettings } from "@/core/render/lib/settings/render-shared-settings";

/** What the application sets for every viewport: the default features and pacing, untimed, unless told. */
export function mockRenderSharedSettings(overrides: Partial<IRenderSharedSettings> = {}): IRenderSharedSettings {
  return {
    features: resolveRendererFeatures(DEFAULT_RENDERER_FEATURE_CHOICE),
    isGpuTimed: false,
    pacing: DEFAULT_RENDER_FRAME_PACING,
    ...overrides,
  };
}
