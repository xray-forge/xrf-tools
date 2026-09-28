import { describe, expect, it } from "@jest/globals";
import { ERendererDebugView, IRendererSettings } from "@xrf/renderer";

import { toAssetRendererSettings } from "@/core/render/lib/settings/asset-renderer-settings";
import { mockRenderSharedSettings } from "@/fixtures/mocks/render.mocks";

describe("toAssetRendererSettings", () => {
  // One asset against a backdrop is no scene to adapt the exposure to, whatever the settings choose for a level.
  it("draws the viewer's choices at the engine's noon scale, with no sky and the exposure held", () => {
    const { features } = mockRenderSharedSettings();
    const settings: IRendererSettings = toAssetRendererSettings(
      { backdrop: 0x353535, isBumped: false, isLit: true, isWireframe: true },
      mockRenderSharedSettings({
        features: { ...features, exposure: { ...features.exposure, isEnabled: true } },
        isGpuTimed: true,
      })
    );

    expect(settings).toMatchObject({
      backdrop: 0x353535,
      debugView: ERendererDebugView.FINAL,
      hemiStrength: 1,
      isBumped: false,
      isGpuTimed: true,
      isLit: true,
      isSkyDrawn: false,
      isTextured: true,
      isWireframe: true,
      tonemapScale: 1,
    });
    expect(settings.features.exposure.isEnabled).toBe(false);
  });
});
