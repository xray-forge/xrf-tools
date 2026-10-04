import { describe, expect, it } from "@jest/globals";

import { ERenderSunShaftsQuality } from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
  ILevelSunShaftsOptions,
  toLevelSunShaftsOptions,
} from "@/core/level/lib/weather/level-sun-shafts-options";

describe("toLevelSunShaftsOptions", () => {
  it("reads stored options back as they were set", () => {
    const options: ILevelSunShaftsOptions = { minimum: 0.3, quality: ERenderSunShaftsQuality.MEDIUM };

    expect(toLevelSunShaftsOptions(JSON.parse(JSON.stringify(options)))).toEqual(options);
  });

  it("holds the minimum to the console's bounds, and defaults what does not read", () => {
    expect(toLevelSunShaftsOptions({ minimum: 0.9, quality: "ultra" })).toEqual({
      minimum: 0.5,
      quality: ERenderSunShaftsQuality.HIGH,
    });
    expect(toLevelSunShaftsOptions(null)).toEqual(DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS);
  });
});
