import { describe, expect, it } from "@jest/globals";

import { toLevelManualWeatherVectorPatch } from "@/core/level/lib/weather/level-weather-vector";

describe("toLevelManualWeatherVectorPatch", () => {
  it("edits as many components as the key holds, zero where the field had fewer", () => {
    expect(toLevelManualWeatherVectorPatch("fogColor", [0.1, 0.2, 0.3, 0.4])).toEqual({ fogColor: [0.1, 0.2, 0.3] });
    expect(toLevelManualWeatherVectorPatch("cloudsColor", [0.5, 0.5])).toEqual({ cloudsColor: [0.5, 0.5, 0, 0] });
  });
});
