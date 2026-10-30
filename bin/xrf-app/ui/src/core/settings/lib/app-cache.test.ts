import { describe, expect, it } from "@jest/globals";

import {
  APP_CACHE_POLICY_CODEC,
  APP_CACHE_POLICY_LIMITS,
  DEFAULT_APP_CACHE_POLICY,
  GIBIBYTE,
} from "@/core/settings/lib/app-cache";

describe("app cache policy", () => {
  it("reads nothing stored as the backend's defaults: a week unused, five gibibytes", () => {
    expect(APP_CACHE_POLICY_CODEC.read(null)).toEqual(DEFAULT_APP_CACHE_POLICY);
    expect(DEFAULT_APP_CACHE_POLICY).toEqual({ sizeLimit: 5 * GIBIBYTE, unusedDays: 7 });
  });

  it("reads what was stored held to its bounds, a missing or broken value as its default", () => {
    expect(APP_CACHE_POLICY_CODEC.read(JSON.stringify({ sizeLimit: 2 * GIBIBYTE, unusedDays: 30 }))).toEqual({
      sizeLimit: 2 * GIBIBYTE,
      unusedDays: 30,
    });
    expect(APP_CACHE_POLICY_CODEC.read(JSON.stringify({ sizeLimit: 1, unusedDays: 900 }))).toEqual({
      sizeLimit: APP_CACHE_POLICY_LIMITS.sizeLimit.min,
      unusedDays: APP_CACHE_POLICY_LIMITS.unusedDays.max,
    });
    expect(APP_CACHE_POLICY_CODEC.read(JSON.stringify({ unusedDays: "soon" }))).toEqual(DEFAULT_APP_CACHE_POLICY);
  });
});
