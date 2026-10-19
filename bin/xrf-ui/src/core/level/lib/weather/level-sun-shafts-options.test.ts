import { describe, expect, it } from "@jest/globals";

import {
  DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
  ILevelSunShaftsOptions,
  toLevelSunShaftsOptions,
} from "@/core/level/lib/weather/level-sun-shafts-options";

describe("toLevelSunShaftsOptions", () => {
  it("reads stored options back as they were set", () => {
    const options: ILevelSunShaftsOptions = { minimum: 0.3, steps: 25 };

    expect(toLevelSunShaftsOptions(JSON.parse(JSON.stringify(options)))).toEqual(options);
  });

  it("holds the minimum and the steps to their bounds, and defaults what does not read", () => {
    expect(toLevelSunShaftsOptions({ minimum: 0.9, steps: 500 })).toEqual({ minimum: 0.5, steps: 64 });
    expect(toLevelSunShaftsOptions({ minimum: 0.1, steps: "ultra" })).toEqual({ minimum: 0.1, steps: 30 });
    expect(toLevelSunShaftsOptions(null)).toEqual(DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS);
  });
});
