import { describe, expect, it } from "@jest/globals";

import { ERenderFoliageMode } from "@/core/ipc/types/xrf-renderer";
import {
  LEVEL_FOLIAGE_LIGHT_KEYS,
  LEVEL_FOLIAGE_MOTION_KEYS,
  resetLevelFoliageKeys,
} from "@/core/level/lib/features/level-foliage-keys";
import { DEFAULT_RENDER_GRASS_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderFoliageSettings } from "@/core/render/lib/settings/render-feature-settings";

const SETTINGS: TRenderFoliageSettings = DEFAULT_RENDER_GRASS_SETTINGS.foliage;

describe("resetLevelFoliageKeys", () => {
  it("splits the foliage between the two popovers, every key one popover's", () => {
    expect([...LEVEL_FOLIAGE_MOTION_KEYS, ...LEVEL_FOLIAGE_LIGHT_KEYS].sort()).toEqual(Object.keys(SETTINGS).sort());
  });

  it("sets the motion back, keeping the light through leaves the view set", () => {
    const foliage: TRenderFoliageSettings = {
      ...SETTINGS,
      mode: ERenderFoliageMode.ENHANCED,
      sssIntensity: SETTINGS.sssIntensity + 0.25,
      treesBend: SETTINGS.treesBend + 0.5,
    };

    expect(resetLevelFoliageKeys({ density: 2, foliage }, SETTINGS, LEVEL_FOLIAGE_MOTION_KEYS)).toEqual({
      density: 2,
      foliage: { ...SETTINGS, sssIntensity: SETTINGS.sssIntensity + 0.25 },
    });
  });

  it("leaves the foliage to the settings once nothing of it differs", () => {
    const foliage: TRenderFoliageSettings = { ...SETTINGS, treesBend: SETTINGS.treesBend + 0.5 };

    expect(resetLevelFoliageKeys({ density: 2, foliage }, SETTINGS, LEVEL_FOLIAGE_MOTION_KEYS)).toEqual({ density: 2 });
    expect(resetLevelFoliageKeys({ radius: 30 }, SETTINGS, LEVEL_FOLIAGE_LIGHT_KEYS)).toEqual({ radius: 30 });
  });
});
