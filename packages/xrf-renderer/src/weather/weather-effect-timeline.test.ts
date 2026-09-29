/// <reference types="node" />

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IWeatherEffectTimeline, toWeatherEffectTimeline } from "#/weather/weather-effect-timeline";
import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";
import { selectWeatherKeyframes, weighWeatherTime } from "#/weather/weather-mixer";

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

/** The pair a forced start blends at a time. */
function toCurrent(time: number): TWeatherKeyframePair {
  const [from, to] = selectWeatherKeyframes(CYCLE, time) as readonly [number, number];

  return [CYCLE[from], CYCLE[to]];
}

function toTimeline(time: number, factor: number): Nullable<IWeatherEffectTimeline> {
  return toWeatherEffectTimeline({ current: toCurrent(time), cycle: CYCLE, effect: EFFECT, factor, name: "fx", time });
}

describe("toWeatherEffectTimeline", () => {
  it("leads in at the weight the clock stands at, plays the effect, and hands the cycle back its next keyframes", () => {
    // Seven in the morning, a sixth of the way from six to noon, at twelve game seconds a real one.
    const timeline: Nullable<IWeatherEffectTimeline> = toTimeline(25_200, 12);

    expect(timeline?.keyframes.map((it: IRendererWeatherKeyframe) => it.time)).toEqual([
      25_188, 25_260, 25_320, 25_380, 25_440,
    ]);
    expect(timeline?.start.map((it: IRendererWeatherKeyframe) => it.time)).toEqual([25_188, 25_260]);
    expect(weighWeatherTime(25_200, [25_188, 25_260])).toBeCloseTo(1 / 6, 10);
    // The lead-in reaches the cycle's noon, the effect's own first keyframe standing in for it.
    expect(timeline?.keyframes[1].fogDistance).toBe(CYCLE[2].fogDistance);
    expect(timeline?.keyframes[2].fogDistance).toBe(70);
    // `WFX_end_desc`: the cycle's noon, at or after the effect's last, and the keyframe after it; the lead-out copies
    // the first a lead-in past the effect's last, which is when the effect ends.
    expect(timeline?.end).toEqual([CYCLE[2], CYCLE[3]]);
    expect(timeline?.keyframes[4].fogDistance).toBe(CYCLE[2].fogDistance);
    expect(timeline?.duration).toBe(25_440 - 25_200);
  });

  it("carries the weight across midnight", () => {
    const timeline: Nullable<IWeatherEffectTimeline> = toTimeline(86_300, 100);
    const keyframes: ReadonlyArray<IRendererWeatherKeyframe> = timeline?.keyframes ?? [];

    expect(keyframes.every((it: IRendererWeatherKeyframe) => it.time >= 0 && it.time < 86_400)).toBe(true);
    // The lead-in sorts last and the cycle's midnight first, and the weight between them is the one the clock was at.
    expect(weighWeatherTime(86_300, [keyframes[keyframes.length - 1].time, keyframes[0].time])).toBeCloseTo(
      (86_300 - 75_600) / 10_800,
      6
    );
  });

  it("plays nothing without the effect's keyframes", () => {
    expect(
      toWeatherEffectTimeline({ current: toCurrent(0), cycle: CYCLE, effect: [], factor: 12, name: "fx", time: 0 })
    ).toBeNull();
  });
});
