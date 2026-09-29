/// <reference types="node" />

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IWeatherEffectTimeline, toWeatherEffectTimeline } from "#/weather/weather-effect-timeline";
import { weighWeatherTime } from "#/weather/weather-mixer";

/** Midnight, six, noon and nine at night. */
const CYCLE: ReadonlyArray<IRendererWeatherKeyframe> = (
  JSON.parse(readFileSync(join(__dirname, "weather-mix.golden.json"), "utf8")) as Array<{
    keyframes: Array<IRendererWeatherKeyframe>;
  }>
)[0].keyframes;

/** An effect a minute a keyframe, its own first replaced by the cycle's next as the engine replaces it. */
const EFFECT: ReadonlyArray<IRendererWeatherKeyframe> = [0, 60, 120].map((time: number) => ({
  ...CYCLE[0],
  fogDistance: 10 + time,
  time,
}));

describe("toWeatherEffectTimeline", () => {
  it("leads in at the weight the clock stands at, plays the effect, and holds the cycle's next until its time", () => {
    // Seven in the morning, a sixth of the way from six to noon, at twelve game seconds a real one.
    const timeline: Nullable<IWeatherEffectTimeline> = toWeatherEffectTimeline({
      cycle: CYCLE,
      effect: EFFECT,
      factor: 12,
      name: "fx_test",
      time: 25_200,
    });

    expect(timeline?.keyframes.map((it: IRendererWeatherKeyframe) => it.time)).toEqual([
      25_188, 25_260, 25_320, 25_380, 25_440, 43_200,
    ]);
    expect(weighWeatherTime(25_200, [25_188, 25_260])).toBeCloseTo(1 / 6, 10);
    // The lead-in reaches the cycle's noon, the effect's own first keyframe standing in for it.
    expect(timeline?.keyframes[1].fogDistance).toBe(CYCLE[2].fogDistance);
    expect(timeline?.keyframes[2].fogDistance).toBe(70);
    expect(timeline?.keyframes[5].fogDistance).toBe(CYCLE[2].fogDistance);
    expect(timeline?.duration).toBe(43_200 - 25_200);
  });

  it("gives the cycle back a lead-in after the effect where its next keyframe is nearer", () => {
    const timeline: Nullable<IWeatherEffectTimeline> = toWeatherEffectTimeline({
      cycle: CYCLE,
      effect: EFFECT,
      factor: 12,
      name: "fx_test",
      time: 43_000,
    });

    // Noon falls inside the lead-out, so the cycle takes over from it as the lead-out ends.
    expect(timeline?.keyframes.slice(1).map((it: IRendererWeatherKeyframe) => it.time)).toEqual([
      43_060, 43_120, 43_180, 43_240,
    ]);
    expect(timeline?.duration).toBe(240);
  });

  it("carries the weight across midnight", () => {
    const timeline: Nullable<IWeatherEffectTimeline> = toWeatherEffectTimeline({
      cycle: CYCLE,
      effect: EFFECT,
      factor: 100,
      name: "fx_test",
      time: 86_300,
    });

    const keyframes: ReadonlyArray<IRendererWeatherKeyframe> = timeline?.keyframes ?? [];

    expect(keyframes.every((it: IRendererWeatherKeyframe) => it.time >= 0 && it.time < 86_400)).toBe(true);
    // The lead-in sorts last and the cycle's midnight first, and the weight between them is the one the clock was at.
    expect(weighWeatherTime(86_300, [keyframes[keyframes.length - 1].time, keyframes[0].time])).toBeCloseTo(
      (86_300 - 75_600) / 10_800,
      6
    );
  });

  it("plays nothing without the effect's keyframes", () => {
    expect(toWeatherEffectTimeline({ cycle: CYCLE, effect: [], factor: 12, name: "fx", time: 0 })).toBeNull();
  });
});
