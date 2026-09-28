import { describe, expect, it } from "@jest/globals";
import {
  DEFAULT_RENDER_FRAME_PACING,
  DEFAULT_RENDERER_FEATURE_CHOICE,
  ERendererDebugView,
  IRendererFeatureSettings,
  IRendererSettings,
  resolveRendererFeatures,
} from "@xrf/renderer";

import { toAssetRendererSettings } from "@/core/render/lib/settings/asset-renderer-settings";

describe("toAssetRendererSettings", () => {
  // One asset against a backdrop is no scene to adapt the exposure to, whatever the settings choose for a level.
  it("draws the viewer's choices at the engine's noon scale, with no sky and the exposure held", () => {
    const features: IRendererFeatureSettings = resolveRendererFeatures(DEFAULT_RENDERER_FEATURE_CHOICE);
    const settings: IRendererSettings = toAssetRendererSettings(
      { backdrop: 0x353535, isBumped: false, isLit: true, isWireframe: true, pacing: DEFAULT_RENDER_FRAME_PACING },
      { ...features, exposure: { ...features.exposure, isEnabled: true } }
    );

    expect(settings).toMatchObject({
      backdrop: 0x353535,
      debugView: ERendererDebugView.FINAL,
      hemiStrength: 1,
      isBumped: false,
      isLit: true,
      isSkyDrawn: false,
      isWireframe: true,
      tonemapScale: 1,
    });
    expect(settings.features.exposure.isEnabled).toBe(false);
  });
});
