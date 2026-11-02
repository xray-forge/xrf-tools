import { describe, expect, it } from "@jest/globals";
import { ELevelObjectKind } from "@xrf/ipc/types/xrf-app-levels";

import {
  DEFAULT_LEVEL_HELPER_OPTIONS,
  isLevelHelperOn,
  toLevelHelperOptions,
} from "@/core/level/lib/objects/level-helper-options";

describe("isLevelHelperOn", () => {
  it("draws a kind as switched, and one never switched as the kind table says", () => {
    const options = {
      ...DEFAULT_LEVEL_HELPER_OPTIONS,
      isShown: true,
      kinds: { [ELevelObjectKind.RESTRICTORS]: false },
    };

    expect(isLevelHelperOn(options, ELevelObjectKind.RESTRICTORS)).toBe(false);
    expect(isLevelHelperOn(options, ELevelObjectKind.ANOMALIES)).toBe(true);
    expect(isLevelHelperOn(options, ELevelObjectKind.PROPS)).toBe(false);
  });
});

describe("toLevelHelperOptions", () => {
  it("starts in the photo mode where nothing was stored", () => {
    expect(toLevelHelperOptions(null)).toEqual(DEFAULT_LEVEL_HELPER_OPTIONS);
  });

  // A kind renamed or dropped since must not stay switched under a name nothing reads.
  it("keeps only the switches it can read, of kinds the table has", () => {
    expect(
      toLevelHelperOptions({
        distanceScale: 99,
        isRestFaded: 1,
        isScreenshotHelped: "no",
        isShown: "yes",
        kinds: { anomalies: false, gone: true, props: 1, restrictors: true },
        occludedStrength: "half",
      })
    ).toEqual({
      distanceScale: 4,
      isRestFaded: false,
      isScreenshotHelped: false,
      isShown: false,
      kinds: { anomalies: false, restrictors: true },
      occludedStrength: DEFAULT_LEVEL_HELPER_OPTIONS.occludedStrength,
    });
    expect(toLevelHelperOptions({ isRestFaded: true }).isRestFaded).toBe(true);
    expect(toLevelHelperOptions({ isScreenshotHelped: true }).isScreenshotHelped).toBe(true);
  });
});
