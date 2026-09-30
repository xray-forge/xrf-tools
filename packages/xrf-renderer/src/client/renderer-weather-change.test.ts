import { describe, expect, it } from "@jest/globals";

import { toRendererWeatherChange } from "#/client/renderer-weather-change";
import { ERendererEngine } from "#/contract/renderer-engine";
import { IRendererWeather } from "#/contract/weather/renderer-weather";

const WEATHER: IRendererWeather = {
  effects: {},
  engine: ERendererEngine.VANILLA,
  keyframes: [],
  modifiers: [],
  rain: null,
  sunTable: null,
  textures: {},
  thunder: null,
  wet: null,
};

describe("toRendererWeatherChange", () => {
  it("sends the whole of a weather after none", () => {
    expect(toRendererWeatherChange(null, WEATHER)).toBe(WEATHER);
  });

  // A keyframe set by hand edits its keyframes alone: the level's thunder and effects cost megabytes to send again.
  it("sends the parts handed over as other objects, and none handed over as the same", () => {
    const keyframes: IRendererWeather["keyframes"] = [];
    const textures: IRendererWeather["textures"] = {};

    expect(toRendererWeatherChange(WEATHER, { ...WEATHER, keyframes, textures })).toEqual({ keyframes, textures });
    expect(toRendererWeatherChange(WEATHER, { ...WEATHER })).toEqual({});
  });
});
