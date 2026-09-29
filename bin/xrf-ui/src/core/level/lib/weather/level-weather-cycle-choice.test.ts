import { describe, expect, it } from "@jest/globals";

import { EWeatherCycleKind } from "@/core/ipc/types/xrf-environment";
import { listLevelWeatherCycles } from "@/core/level/lib/weather/level-weather-cycle-choice";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";
import { mockLevelWeatherCycle, mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";

describe("listLevelWeatherCycles", () => {
  it("lists the level's cycles first with the states that play them, then the game's others by name", () => {
    const description = mockLevelWeatherDescription({
      cycles: [
        { file: "", findings: 0, keyframes: 24, kind: EWeatherCycleKind.CYCLE, name: "indoor" },
        { file: "", findings: 1, keyframes: 24, kind: EWeatherCycleKind.CYCLE, name: "default_clear" },
        { file: "", findings: 0, keyframes: 24, kind: EWeatherCycleKind.CYCLE, name: "default_cloudy" },
        { file: "", findings: 0, keyframes: 3, kind: EWeatherCycleKind.EFFECT, name: "fx_blowout_day" },
      ],
      offered: [mockLevelWeatherCycle()],
      weather: {
        key: "dynamic_default",
        level: "zaton",
        options: [{ cycle: "default_clear", graph: "dynamic_default", state: "clear" }],
      },
    });

    expect(listLevelWeatherCycles(description)).toEqual([
      { findings: 0, isOffered: true, keyframes: 2, name: "default_clear", states: ["dynamic_default: clear"] },
      { findings: 0, isOffered: false, keyframes: 24, name: "default_cloudy", states: [] },
      { findings: 0, isOffered: false, keyframes: 24, name: "indoor", states: [] },
    ]);
  });
});

describe("formatLevelWeatherTime", () => {
  it("says a time of day as a keyframe names one, wrapped into the day", () => {
    expect(formatLevelWeatherTime(0)).toBe("00:00");
    expect(formatLevelWeatherTime(45_296, true)).toBe("12:34:56");
    expect(formatLevelWeatherTime(86_400 + 60)).toBe("00:01");
  });
});
