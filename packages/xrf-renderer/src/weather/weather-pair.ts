import { Maybe, Nullable } from "@xrf/types";

import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";
import { selectWeatherKeyframes } from "#/weather/weather-mixer";

/**
 * `Current[0]` and `Current[1]`: the two keyframes the engine blends between, kept from frame to frame. The pair moves
 * on only once the clock passes its second keyframe, taking the next from whatever the weather plays by then, so a
 * weather that changes under it blends in over the rest of the span rather than at once (`SetWeather(name, false)`).
 */
export class WeatherPair {
  private pair: Nullable<TWeatherKeyframePair> = null;

  /** The pair, or null before anything was selected. */
  public get current(): Nullable<TWeatherKeyframePair> {
    return this.pair;
  }

  /**
   * `SelectEnvs` on a forced start: the keyframes around the time, whatever was blended before.
   *
   * @param keyframes - Sorted by time.
   * @param time - Seconds since midnight.
   */
  public force(keyframes: ReadonlyArray<IRendererWeatherKeyframe>, time: number): void {
    const selected: Nullable<readonly [number, number]> = selectWeatherKeyframes(keyframes, time);

    this.pair = selected ? [keyframes[selected[0]], keyframes[selected[1]]] : null;
  }

  /**
   * @param pair - What to blend between from now on, as `StopWFX` hands the cycle back its keyframes.
   */
  public set(pair: TWeatherKeyframePair): void {
    this.pair = pair;
  }

  /** Forgets the pair, so the next advance selects afresh. */
  public reset(): void {
    this.pair = null;
  }

  /**
   * `SelectEnvs` once started: past the second keyframe, it becomes the first and the next is the first at or after
   * the time; across midnight, only while the time is between the two.
   *
   * @param keyframes - What plays now, sorted by time.
   * @param time - Seconds since midnight.
   */
  public advance(keyframes: ReadonlyArray<IRendererWeatherKeyframe>, time: number): void {
    const { pair } = this;

    if (!pair) {
      return this.force(keyframes, time);
    }

    const [from, to] = pair;
    const isPast: boolean = from.time > to.time ? time > to.time && time < from.time : time > to.time;

    if (isPast) {
      this.pair = [to, WeatherPair.selectNext(keyframes, time) ?? to];
    }
  }

  /**
   * `SelectEnv`: the first keyframe at or after a time, the first of the day past the last.
   *
   * @param keyframes - Sorted by time.
   * @param time - Seconds since midnight.
   * @returns The keyframe, or undefined for none.
   */
  public static selectNext(
    keyframes: ReadonlyArray<IRendererWeatherKeyframe>,
    time: number
  ): Maybe<IRendererWeatherKeyframe> {
    return keyframes.find((keyframe: IRendererWeatherKeyframe) => keyframe.time >= time) ?? keyframes[0];
  }
}
