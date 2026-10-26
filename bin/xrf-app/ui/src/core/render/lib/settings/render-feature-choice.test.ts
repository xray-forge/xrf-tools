import { describe, expect, it } from "@jest/globals";

import {
  DEFAULT_RENDER_FEATURE_CHOICE,
  IRenderFeatureChoice,
  listUnkeptRenderSettings,
  resolveRenderFeatures,
  toRenderFeatureChoice,
} from "@/core/render/lib/settings/render-feature-choice";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";

/**
 * @param cascades - Cascade widths as stored.
 * @returns The widths the stored choice keeps, if any.
 */
function toCascades(cascades: unknown): unknown {
  return toRenderFeatureChoice({ overrides: { shadows: { cascades } }, preset: ERenderPreset.BASE }).overrides.shadows
    ?.cascades;
}

// What a stored choice parses to is all that stands between an edited or stale value and the renderer.
describe("render feature choice", () => {
  it("holds stored numbers to their bounds and drops what the features do not take", () => {
    const choice: IRenderFeatureChoice = toRenderFeatureChoice({
      overrides: {
        grass: { density: 7, radius: "far", unknown: 1 },
        shadows: { filter: 2.6, resolution: Number.POSITIVE_INFINITY, bias: Number.NaN },
        unknownGroup: { value: 1 },
      },
      preset: ERenderPreset.EDITING,
    });

    expect(choice).toEqual({
      overrides: { grass: { density: 0.99 }, shadows: { filter: 3 } },
      preset: ERenderPreset.EDITING,
    });
  });

  it("takes a run of widths only whole, each held to its bounds", () => {
    expect(toCascades([0.5, 40, 5000])).toEqual([1, 40, 2000]);
    expect(toCascades([])).toBeUndefined();
    expect(toCascades([20, 40, 80, 160, 320])).toBeUndefined();
    expect(toCascades([20, 0])).toBeUndefined();
    expect(toCascades([20, -1])).toBeUndefined();
    expect(toCascades([20, Number.NaN])).toBeUndefined();
    expect(toCascades("20,40")).toBeUndefined();
  });

  it("falls back to the default for a choice that is not one, or names no preset", () => {
    expect(toRenderFeatureChoice(null)).toBe(DEFAULT_RENDER_FEATURE_CHOICE);
    expect(toRenderFeatureChoice([1, 2])).toBe(DEFAULT_RENDER_FEATURE_CHOICE);
    expect(toRenderFeatureChoice({ preset: "ultra" })).toEqual(DEFAULT_RENDER_FEATURE_CHOICE);
  });

  it("names every setting it did not keep as given", () => {
    const sent: object = {
      reflections: { quality: "medium", steps: 500, traceInterval: 2 },
      shadows: { cascades: [20, 40] },
    };
    const choice: IRenderFeatureChoice = toRenderFeatureChoice({ overrides: sent, preset: ERenderPreset.BASE });

    expect(listUnkeptRenderSettings(sent, choice.overrides)).toEqual(["reflections.quality", "reflections.steps"]);
    expect(listUnkeptRenderSettings(choice.overrides, choice.overrides)).toEqual([]);
  });

  it("parses every preset's own features back unchanged", () => {
    for (const preset of Object.values(ERenderPreset)) {
      const choice: IRenderFeatureChoice = toRenderFeatureChoice({ overrides: RENDER_PRESETS[preset], preset });

      expect(resolveRenderFeatures(choice)).toEqual(RENDER_PRESETS[preset]);
    }
  });
});
