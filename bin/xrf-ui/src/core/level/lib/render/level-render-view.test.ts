import { describe, expect, it } from "@jest/globals";

import {
  ERenderAntialiasing,
  ERenderDebugView,
  ERenderSunShaftsQuality,
  ERenderSurfaceColor,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_LEVEL_LOD_OPTIONS } from "@/core/level/lib/lod/level-lod-options";
import { OPENXRAY_LEVEL_LOOK } from "@/core/level/lib/look/level-look";
import { ILevelViewOptionsInputs, toLevelViewOptions } from "@/core/level/lib/render/level-render-view";
import { ELevelShading } from "@/core/level/lib/view/level-shading";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS } from "@/core/level/lib/weather/level-sun-shafts-options";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";
import { mockRenderFeatures } from "@/fixtures/mocks/render.mocks";

function toInputs(overrides: Partial<ILevelViewOptionsInputs> = {}): ILevelViewOptionsInputs {
  return {
    shading: ELevelShading.FINAL,
    features: mockRenderFeatures(),
    hemiStrength: 0.8,
    lod: DEFAULT_LEVEL_LOD_OPTIONS,
    look: OPENXRAY_LEVEL_LOOK,
    options: DEFAULT_LEVEL_VIEW_OPTIONS,
    renderHeight: null,
    sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
    view: mockLevelFeatureOptions(),
    ...overrides,
  };
}

describe("toLevelViewOptions", () => {
  it("steps the sunshafts as the sun's popover sets them", () => {
    const sunShafts = { minimum: 0.25, quality: ERenderSunShaftsQuality.LOW };

    expect(toLevelViewOptions(toInputs({ sunShafts })).features.sunShafts).toEqual(sunShafts);
  });

  it("composites the wall marks while the toolbar shows them", () => {
    expect(toLevelViewOptions(toInputs()).show.isWallmarked).toBe(true);
    expect(
      toLevelViewOptions(toInputs({ options: { ...DEFAULT_LEVEL_VIEW_OPTIONS, isWallmarked: false } })).show
        .isWallmarked
    ).toBe(false);
  });

  it("darkens the ambient by the baked hemisphere only while the toolbar bakes it", () => {
    expect(toLevelViewOptions(toInputs()).features.hemiStrength).toBe(0.8);
    expect(
      toLevelViewOptions(toInputs({ options: { ...DEFAULT_LEVEL_VIEW_OPTIONS, isBaked: false } })).features.hemiStrength
    ).toBe(0);
  });

  it("narrows the settings' features by the toolbar's toggles", () => {
    const options: RenderViewOptions = toLevelViewOptions(
      toInputs({
        features: mockRenderFeatures({ antialiasing: ERenderAntialiasing.TAA }),
        options: { ...DEFAULT_LEVEL_VIEW_OPTIONS, isAntialiased: false, isGrassy: false, isShadowed: false },
      })
    );

    expect(options.features.antialiasing).toBe(ERenderAntialiasing.NONE);
    expect(options.features.grass.isEnabled).toBe(false);
    expect(options.features.shadows.isEnabled).toBe(false);
  });

  it("draws at the height the settings ask, and at the look's exposure", () => {
    const options: RenderViewOptions = toLevelViewOptions(toInputs({ renderHeight: 1080 }));

    expect(options.output.renderHeight).toBe(1080);
    expect(options.features.exposure).toEqual(OPENXRAY_LEVEL_LOOK.exposure);
  });

  it("shows the shading chosen as a surface colour and a picture", () => {
    const shader: RenderViewOptions = toLevelViewOptions(toInputs({ shading: ELevelShading.SHADER }));
    const normal: RenderViewOptions = toLevelViewOptions(toInputs({ shading: ELevelShading.NORMAL }));

    expect([shader.mode.surfaceColor, shader.mode.debugView]).toEqual([
      ERenderSurfaceColor.SHADER,
      ERenderDebugView.FINAL,
    ]);
    expect([normal.mode.surfaceColor, normal.mode.debugView]).toEqual([
      ERenderSurfaceColor.TEXTURED,
      ERenderDebugView.NORMAL,
    ]);
  });
});
