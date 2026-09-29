import { Nullable } from "@xrf/types";

import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { toWeatherTimeOfDay, WEATHER_DAY_LENGTH } from "#/weather/weather-day";
import { IWeatherEffectStart } from "#/weather/weather-effect-start";
import { selectWeatherKeyframes } from "#/weather/weather-mixer";

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
  /** Game seconds from its start until the cycle takes over again. */
  duration: number;
}

/**
 * `SetWeatherFX`: the cycle's two keyframes around the start, carried on at the weight they stand at into the second
 * one a lead-in later; the effect's keyframes after that, its first replaced by that second one as the engine
 * replaces it; then the cycle's keyframe at or after the effect's last, a lead-in on, held until its own time.
 *
 * @param start - The effect, the cycle, and when and how fast it starts.
 * @returns Its timeline, or null for an effect or a cycle with no keyframes.
 */
export function toWeatherEffectTimeline(start: IWeatherEffectStart): Nullable<IWeatherEffectTimeline> {
  const { name, effect, cycle, time, factor } = start;
  const selected: Nullable<readonly [number, number]> = selectWeatherKeyframes(cycle, time);

  if (!selected || !effect.length) {
    return null;
  }

  const a: IRendererWeatherKeyframe = cycle[selected[0]];
  const b: IRendererWeatherKeyframe = cycle[selected[1]];
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
  // `SelectEnv`: the cycle's first keyframe at or after the effect's last.
  const back: IRendererWeatherKeyframe =
    cycle[(selectWeatherKeyframes(cycle, toWeatherTimeOfDay(last)) ?? selected)[1]];
  const end: number = last + rewind;
  const held: number = toElapsed(toWeatherTimeOfDay(last), back.time);
  const keyframes: Array<IRendererWeatherKeyframe> = [leadIn, ...played, { ...back, time: toWeatherTimeOfDay(end) }];

  // Played on from where the effect left it, the cycle holds its keyframe until the clock reaches it.
  if (held > rewind) {
    keyframes.push({ ...back, time: back.time });
  }

  return {
    duration: Math.max(held, rewind) + (last - time),
    keyframes: keyframes.sort((first, second) => first.time - second.time),
    name,
  };
}

/** `TimeDiff`: seconds from one time of day to the next, around midnight where it comes first. */
function toElapsed(from: number, to: number): number {
  const a: number = toWeatherTimeOfDay(from);
  const b: number = toWeatherTimeOfDay(to);

  return a > b ? WEATHER_DAY_LENGTH - a + b : b - a;
}
