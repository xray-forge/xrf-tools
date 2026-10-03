import { describe, expect, it } from "@jest/globals";

import { ERenderAntialiasing, ERenderDebugView, RenderViewOptions } from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_LEVEL_LOD_OPTIONS } from "@/core/level/lib/lod/level-lod-options";
import { OPENXRAY_LEVEL_LOOK } from "@/core/level/lib/look/level-look";
import { ILevelViewOptionsInputs, toLevelViewOptions } from "@/core/level/lib/render/level-render-view";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { mockRenderFeatures } from "@/fixtures/mocks/render.mocks";

function toInputs(overrides: Partial<ILevelViewOptionsInputs> = {}): ILevelViewOptionsInputs {
  return {
    debugView: ERenderDebugView.FINAL,
    features: mockRenderFeatures(),
    hemiStrength: 0.8,
    lod: DEFAULT_LEVEL_LOD_OPTIONS,
    look: OPENXRAY_LEVEL_LOOK,
    options: DEFAULT_LEVEL_VIEW_OPTIONS,
    renderHeight: null,
    view: mockLevelFeatureOptions(),
    ...overrides,
  };
}

describe("toLevelViewOptions", () => {
  it("composites the wall marks while the toolbar shows them", () => {
    expect(toLevelViewOptions(toInputs()).isWallmarked).toBe(true);
    expect(
      toLevelViewOptions(toInputs({ options: { ...DEFAULT_LEVEL_VIEW_OPTIONS, isWallmarked: false } })).isWallmarked
    ).toBe(false);
  });

  it("darkens the ambient by the baked hemisphere only while the toolbar bakes it", () => {
    expect(toLevelViewOptions(toInputs()).hemiStrength).toBe(0.8);
    expect(
      toLevelViewOptions(toInputs({ options: { ...DEFAULT_LEVEL_VIEW_OPTIONS, isBaked: false } })).hemiStrength
    ).toBe(0);
  });

  it("narrows the settings' features by the toolbar's toggles", () => {
    const options: RenderViewOptions = toLevelViewOptions(
      toInputs({
        features: mockRenderFeatures({ antialiasing: ERenderAntialiasing.TAA }),
        options: { ...DEFAULT_LEVEL_VIEW_OPTIONS, isAntialiased: false, isGrassy: false, isShadowed: false },
      })
    );

    expect(options.antialiasing).toBe(ERenderAntialiasing.NONE);
    expect(options.grass.isEnabled).toBe(false);
    expect(options.shadows.isEnabled).toBe(false);
  });

  it("draws at the height the settings ask, and at the look's exposure", () => {
    const options: RenderViewOptions = toLevelViewOptions(toInputs({ renderHeight: 1080 }));

    expect(options.renderHeight).toBe(1080);
    expect(options.exposure).toEqual(OPENXRAY_LEVEL_LOOK.exposure);
  });
});
