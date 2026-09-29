/// <reference types="node" />

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "@jest/globals";

import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { WeatherPair } from "#/weather/weather-pair";

/** Midnight, six, noon and nine at night. */
const CYCLE: ReadonlyArray<IRendererWeatherKeyframe> = (
  JSON.parse(readFileSync(join(__dirname, "weather-mix.golden.json"), "utf8")) as Array<{
    keyframes: Array<IRendererWeatherKeyframe>;
  }>
)[0].keyframes;

/** The same day with every keyframe an hour later, as another cycle. */
const OTHER: ReadonlyArray<IRendererWeatherKeyframe> = CYCLE.map((it: IRendererWeatherKeyframe) => ({
  ...it,
  fogDistance: 1,
  time: it.time + 3_600,
}));

function toTimes(pair: WeatherPair): Array<number> {
  return pair.current?.map((it: IRendererWeatherKeyframe) => it.time) ?? [];
}

describe("WeatherPair", () => {
  it("selects the keyframes around the time on a forced start, and moves on only past the second", () => {
    const pair: WeatherPair = new WeatherPair();

    pair.advance(CYCLE, 30_000);

    expect(toTimes(pair)).toEqual([21_600, 43_200]);

    pair.advance(CYCLE, 43_200);

    expect(toTimes(pair)).toEqual([21_600, 43_200]);

    pair.advance(CYCLE, 43_201);

    expect(toTimes(pair)).toEqual([43_200, 75_600]);
  });

  // `SetWeather(name, false)`: the pair being blended stays, and the next keyframe comes from the weather playing then.
  it("keeps blending the pair it has when the weather changes under it, taking the next from the new one", () => {
    const pair: WeatherPair = new WeatherPair();

    pair.advance(CYCLE, 30_000);
    pair.advance(OTHER, 40_000);

    expect(toTimes(pair)).toEqual([21_600, 43_200]);
    expect(pair.current?.[1].fogDistance).toBe(CYCLE[2].fogDistance);

    pair.advance(OTHER, 43_300);

    expect(toTimes(pair)).toEqual([43_200, 46_800]);
    expect(pair.current?.[1].fogDistance).toBe(1);
  });

  it("moves on across midnight only once the time is between the pair's two", () => {
    const pair: WeatherPair = new WeatherPair();

    pair.advance(CYCLE, 80_000);

    expect(toTimes(pair)).toEqual([75_600, 0]);

    pair.advance(CYCLE, 86_000);

    expect(toTimes(pair)).toEqual([75_600, 0]);

    pair.advance(CYCLE, 100);

    expect(toTimes(pair)).toEqual([0, 21_600]);
  });

  it("stays on the one keyframe of a cycle of one, whatever the time", () => {
    const pair: WeatherPair = new WeatherPair();
    const manual: ReadonlyArray<IRendererWeatherKeyframe> = [CYCLE[2]];

    pair.advance(manual, 1_000);
    pair.advance(manual, 60_000);

    expect(pair.current).toEqual([CYCLE[2], CYCLE[2]]);
  });
});
