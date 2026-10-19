import { describe, expect, it } from "@jest/globals";

import { ERenderAmbientOcclusionMethod, ERenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_RENDER_FEATURE_CHOICE,
  resolveRenderFeatures,
  toRenderFeatureChoice,
} from "@/core/render/lib/settings/render-feature-choice";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
import { ERenderSettingKind, isRenderSettingField } from "@/core/render/lib/settings/render-setting-field";

/** Each rated number of the schema, its place and its recommended range. */
function listRated(
  node: object,
  path: ReadonlyArray<string> = []
): Array<[ReadonlyArray<string>, readonly [number, number]]> {
  if (isRenderSettingField(node)) {
    return node.kind === ERenderSettingKind.NUMBER && node.recommended ? [[path, node.recommended]] : [];
  }

  return Object.entries(node).flatMap(([key, child]) =>
    typeof child === "object" && child !== null ? listRated(child, [...path, key]) : []
  );
}

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
    const rated = listRated(RENDER_FEATURE_SCHEMA);

    expect(rated.length).toBeGreaterThan(0);

    for (const preset of [ERenderPreset.BASE, ERenderPreset.ENHANCED]) {
      for (const [path, [low, high]] of rated) {
        const value: unknown = path.reduce<unknown>(
          (node: unknown, key: string) => (node as Record<string, unknown>)[key],
          RENDER_PRESETS[preset]
        );

        expect({
          path: path.join("."),
          preset,
          within: typeof value === "number" && value >= low && value <= high,
        }).toEqual({ path: path.join("."), preset, within: true });
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
