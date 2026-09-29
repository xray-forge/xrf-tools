import { describe, expect, it } from "@jest/globals";

import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { DEFAULT_LEVEL_MANUAL_WEATHER } from "@/core/level/lib/weather/level-manual-weather";
import { toLevelManualWeatherLtx } from "@/core/level/lib/weather/level-manual-weather-ltx";

describe("toLevelManualWeatherLtx", () => {
  it("writes the keyframe as a section named by its time, with the keys vanilla reads", () => {
    const text: string = toLevelManualWeatherLtx({
      engine: EXrayEngine.VANILLA,
      manual: { ...DEFAULT_LEVEL_MANUAL_WEATHER, cloudsColor: [0.5, 0.5, 0.5, 0.8] },
      time: 45_296,
    });
    const lines: Array<string> = text.split("\n");

    expect(lines[0]).toBe("[12:34:56]");
    expect(text).toContain("sky_texture      = sky\\sky_7_cube\n");
    expect(text).toContain("clouds_color     = 0.5, 0.5, 0.5, 0.8, 2\n");
    expect(text).toContain("sun_altitude     = -68.999985\n");
    expect(text).toContain("fog_color        = 0.304609, 0.328138, 0.367354\n");
    expect(text).toContain("trees_wave       = 0.1, 0.01, 0.11\n");
    expect(text).not.toContain("tree_amplitude_intensity");
  });

  // Monolith stands its sun by the sun table and reads Call of Chernobyl's one tree key.
  it("leaves out what an extended engine does not read", () => {
    const text: string = toLevelManualWeatherLtx({
      engine: EXrayEngine.EXTENDED,
      manual: DEFAULT_LEVEL_MANUAL_WEATHER,
      time: 0,
    });

    expect(text.startsWith("[00:00:00]\n")).toBe(true);
    expect(text).toMatch(/tree_amplitude_intensity = 0\.005\n/);
    expect(text).not.toMatch(/sun_altitude|sun_longitude|trees_|clouds_rotation/);
  });
});
