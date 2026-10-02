import { describe, expect, it } from "@jest/globals";

import { WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import {
  DEFAULT_LEVEL_MANUAL_WEATHER,
  readLevelManualWeather,
  toLevelManualDescriptor,
  toLevelManualSun,
  toLevelManualWeather,
} from "@/core/level/lib/weather/level-manual-weather";

describe("level manual weather", () => {
  // `default_clear`'s noon: `setHP(-69, -30)` stands the light travelling down at thirty degrees.
  it("stands the sun as setHP does, and reads the same angles back out of the direction", () => {
    const keyframe: WeatherDescriptor = toLevelManualDescriptor(DEFAULT_LEVEL_MANUAL_WEATHER, 43_200);
    const [x, y, z] = (keyframe.sunDirection ?? [0, 0, 0]).map((it) => it ?? 0);

    expect(y).toBeCloseTo(-0.5, 5);
    expect(Math.hypot(x, y, z)).toBeCloseTo(1, 5);
    expect(keyframe.skyTextureEnv).toBe("sky\\sky_7_cube#small");
    expect(keyframe.skyRotation).toBeCloseTo(0);

    const back = toLevelManualWeather(keyframe);

    expect(back.sunAltitude).toBeCloseTo(DEFAULT_LEVEL_MANUAL_WEATHER.sunAltitude, 4);
    expect(back.sunLongitude).toBeCloseTo(DEFAULT_LEVEL_MANUAL_WEATHER.sunLongitude, 4);
    expect({ ...back, sunAltitude: 0, sunLongitude: 0 }).toEqual({
      ...DEFAULT_LEVEL_MANUAL_WEATHER,
      sunAltitude: 0,
      sunLongitude: 0,
    });
  });

  it("clamps the rain to a unit, as the engine does", () => {
    expect(toLevelManualDescriptor({ ...DEFAULT_LEVEL_MANUAL_WEATHER, rainDensity: 3 }, 0).rainDensity).toBe(1);
  });

  it("stands a level's own sun by the angles its direction comes to", () => {
    const direction: WeatherDescriptor["sunDirection"] = toLevelManualDescriptor(
      DEFAULT_LEVEL_MANUAL_WEATHER,
      0
    ).sunDirection;
    const [x, y, z] = (direction ?? [0, 0, 0]).map((it) => it ?? 0);
    const sun = toLevelManualSun({ x: x * 4, y: y * 4, z: z * 4 });

    expect(sun?.sunAltitude).toBeCloseTo(-69, 4);
    expect(sun?.sunLongitude).toBeCloseTo(-30, 4);
    expect(toLevelManualSun(null)).toBeNull();
    expect(toLevelManualSun({ x: 0, y: 0, z: 0 })).toBeNull();
  });

  it("reads back only a keyframe whose every field reads as what it holds there", () => {
    const stored: unknown = JSON.parse(JSON.stringify(DEFAULT_LEVEL_MANUAL_WEATHER));

    expect(readLevelManualWeather(stored)).toEqual(DEFAULT_LEVEL_MANUAL_WEATHER);
    expect(readLevelManualWeather({ ...DEFAULT_LEVEL_MANUAL_WEATHER, farPlane: "far" })).toBeNull();
    expect(readLevelManualWeather({ ...DEFAULT_LEVEL_MANUAL_WEATHER, fogColor: [1, 1, Infinity] })).toBeNull();
    expect(readLevelManualWeather({ ...DEFAULT_LEVEL_MANUAL_WEATHER, windVelocity: undefined })).toBeNull();
    expect(readLevelManualWeather("sky")).toBeNull();
  });
});
