import { describe, expect, it } from "@jest/globals";

import { ERenderAmbientOcclusionMethod, ERenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_RENDER_FEATURE_CHOICE,
  resolveRenderFeatures,
  toRenderFeatureChoice,
} from "@/core/render/lib/settings/render-feature-choice";
import {
  listRenderNumberFieldPaths,
  readRenderFeatureNumber,
  TRenderFeaturePath,
} from "@/core/render/lib/settings/render-feature-path";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
import { IRenderNumberField } from "@/core/render/lib/settings/render-setting-field";

describe("render presets", () => {
  it("starts a fresh install on Base", () => {
    expect(DEFAULT_RENDER_FEATURE_CHOICE.preset).toBe(ERenderPreset.BASE);
  });

  // A preset value past a bound would be held to it the moment it is stored back, and so never be what it says.
  it("sets every feature within the bounds a stored choice is held to", () => {
    for (const preset of Object.values(ERenderPreset)) {
      expect(resolveRenderFeatures(toRenderFeatureChoice({ overrides: RENDER_PRESETS[preset], preset }))).toEqual(
        RENDER_PRESETS[preset]
      );
    }
  });

  // A preset that painted its own value amber would say xrf advises against what xrf ships; Editing trades looks for
  // responsiveness on purpose and may go below.
  it("sets every rated value of Base and Enhanced within the range it recommends", () => {
    const rated: Array<[IRenderNumberField, TRenderFeaturePath]> = [
      ...listRenderNumberFieldPaths(RENDER_FEATURE_SCHEMA),
    ].filter(([field]) => field.recommended);

    expect(rated.length).toBeGreaterThan(0);

    for (const preset of [ERenderPreset.BASE, ERenderPreset.ENHANCED]) {
      for (const [field, path] of rated) {
        const [low, high] = field.recommended ?? [0, 0];
        const value: number = readRenderFeatureNumber(RENDER_PRESETS[preset], path) ?? Number.NaN;

        expect({ path: path.join("."), preset, within: value >= low && value <= high }).toEqual({
          path: path.join("."),
          preset,
          within: true,
        });
      }
    }
  });

  // A preset painted red would ship a value measured to cost more than a lower one for the same picture.
  it("sets no value of a preset past the end measured to be wasted", () => {
    const measured: Array<[IRenderNumberField, TRenderFeaturePath]> = [
      ...listRenderNumberFieldPaths(RENDER_FEATURE_SCHEMA),
    ].filter(([field]) => field.wastedAbove !== undefined);

    expect(measured.length).toBeGreaterThan(0);

    for (const preset of Object.values(ERenderPreset)) {
      for (const [field, path] of measured) {
        const value: number = readRenderFeatureNumber(RENDER_PRESETS[preset], path) ?? Number.NaN;

        expect({ path: path.join("."), preset, value }).toEqual({
          path: path.join("."),
          preset,
          value: Math.min(value, field.wastedAbove ?? value),
        });
      }
    }
  });

  it("draws Enhanced at its best with TAA, short of ray reconstruction and traced occlusion", () => {
    const enhanced = RENDER_PRESETS[ERenderPreset.ENHANCED];

    expect(enhanced.antialiasing).toBe(ERenderAntialiasing.TAA);
    expect(enhanced.upscaling.isRayReconstructed).toBe(false);
    expect(enhanced.ambientOcclusion.method).not.toBe(ERenderAmbientOcclusionMethod.RTAO);
  });
});
