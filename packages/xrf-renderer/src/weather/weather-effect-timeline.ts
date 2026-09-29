import { Nullable } from "@xrf/types";

import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { toWeatherTimeOfDay, WEATHER_DAY_LENGTH } from "#/weather/weather-day";
import { IWeatherEffectStart } from "#/weather/weather-effect-start";
import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";
import { WeatherPair } from "#/weather/weather-pair";

/** `WFX_TRANS_TIME`: real seconds an effect takes to lead in, and to lead back into the cycle. */
const TRANSITION: number = 5;

/** `EPS`, under which a span is none. */
const EPS: number = 0.00001;

/**
 * A weather effect laid over the cycle from the time it started, as `SetWeatherFX` lays it.
 */
export interface IWeatherEffectTimeline {
  name: string;
  /** Sorted by time of day: the lead-in, the effect's own keyframes, and the way back into the cycle. */
  keyframes: ReadonlyArray<IRendererWeatherKeyframe>;
  /** The pair it starts blending: the lead-in and the cycle's next keyframe, `C0` and `C1`. */
  start: TWeatherKeyframePair;
  /** The pair it hands the cycle back as it ends, `WFX_end_desc`. */
  end: TWeatherKeyframePair;
  /** Game seconds from its start until the cycle takes over again, `wfx_time`. */
  duration: number;
}

/**
 * `SetWeatherFX`: the cycle's two keyframes around the start, carried on at the weight they stand at into the second
 * one a lead-in later; the effect's keyframes after that, its first replaced by that second one as the engine
 * replaces it; then the cycle's keyframe at or after the effect's last, a lead-in on, from which the cycle takes over
 * with it and the one after.
 *
 * @param start - The effect, the cycle, and when and how fast it starts.
 * @returns Its timeline, or null for an effect or a cycle with no keyframes.
 */
export function toWeatherEffectTimeline(start: IWeatherEffectStart): Nullable<IWeatherEffectTimeline> {
  const { name, effect, cycle, current, time, factor } = start;

  if (!effect.length || !cycle.length) {
    return null;
  }

  const [a, b] = current;
  const rewind: number = TRANSITION * factor;
  const begin: number = time + rewind;
  const toNext: number = toElapsed(time, b.time);
  const length: number = toElapsed(a.time, b.time);
  // The first keyframe moved back so the weight at the start stays what it was.
  const leadIn: IRendererWeatherKeyframe = {
    ...a,
    time: toWeatherTimeOfDay(toNext < EPS ? time : time - ((rewind / toNext) * length - rewind)),
  };
  const played: Array<IRendererWeatherKeyframe> = [
    { ...b, time: toWeatherTimeOfDay(begin) },
    ...effect.slice(1).map((keyframe: IRendererWeatherKeyframe) => ({
      ...keyframe,
      time: toWeatherTimeOfDay(begin + keyframe.time),
    })),
  ];
  const last: number = effect.length > 1 ? begin + effect[effect.length - 1].time : begin;
  // `SelectEnv` twice: the cycle's first keyframe at or after the effect's last, and the one after that.
  const back: IRendererWeatherKeyframe = WeatherPair.selectNext(cycle, toWeatherTimeOfDay(last)) ?? b;
  const after: IRendererWeatherKeyframe = WeatherPair.selectNext(cycle, toWeatherTimeOfDay(back.time + 0.5)) ?? back;
  const keyframes: Array<IRendererWeatherKeyframe> = [
    leadIn,
    ...played,
    { ...back, time: toWeatherTimeOfDay(last + rewind) },
  ];

  return {
    duration: last + rewind - time,
    end: [back, after],
    keyframes: [...keyframes].sort((first, second) => first.time - second.time),
    name,
    start: [leadIn, played[0]],
  };
}

/** `TimeDiff`: seconds from one time of day to the next, around midnight where it comes first. */
function toElapsed(from: number, to: number): number {
  const a: number = toWeatherTimeOfDay(from);
  const b: number = toWeatherTimeOfDay(to);

  return a > b ? WEATHER_DAY_LENGTH - a + b : b - a;
}
