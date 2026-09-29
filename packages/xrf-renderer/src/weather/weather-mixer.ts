import { Nullable } from "@xrf/types";

import { TRendererVector } from "#/contract/renderer-vector";
import { ERendererWeatherEngine } from "#/contract/weather/renderer-weather-engine";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IDynamicSun, toDynamicSun } from "#/weather/dynamic-sun";
import { toSunTableDirection } from "#/weather/sun-table-direction";
import { toWeatherTimeOfDay, WEATHER_DAY_LENGTH } from "#/weather/weather-day";
import { IWeatherMix } from "#/weather/weather-mix";
import { EWeatherSun, TWeatherSun } from "#/weather/weather-sun";

/** `EPS`, what `TimeWeight` takes a zero span by. */
const EPS: number = 0.00001;

/** Where an authored sun stands for a keyframe that stands none. */
const DOWN: TRendererVector = [0, -1, 0];

/**
 * What a cycle is mixed from: `CEnvironment::lerp` without modifiers.
 */
export interface IWeatherMixer {
  /** Sorted by time. */
  keyframes: ReadonlyArray<IRendererWeatherKeyframe>;
  engine: ERendererWeatherEngine;
  sun: TWeatherSun;
}

/**
 * `CEnvDescriptorMixer::lerp` of the keyframes around a time of day.
 *
 * @param mixer - What is mixed.
 * @param at - Seconds, wrapped into the day.
 * @returns The mix, or null for a cycle without keyframes.
 */
export function mixWeather(mixer: IWeatherMixer, at: number): Nullable<IWeatherMix> {
  const { keyframes, engine, sun } = mixer;
  const time: number = toWeatherTimeOfDay(at);
  const selected: Nullable<readonly [number, number]> = selectWeatherKeyframes(keyframes, time);

  if (!selected) {
    return null;
  }

  const a: IRendererWeatherKeyframe = keyframes[selected[0]];
  const b: IRendererWeatherKeyframe = keyframes[selected[1]];
  const f: number = weighWeatherTime(time, [a.time, b.time]);

  function scalar(from: number, to: number): number {
    return (1 - f) * from + f * to;
  }

  function vector<T extends ReadonlyArray<number>>(from: T, to: T): T {
    return from.map((value: number, index: number) => scalar(value, to[index])) as unknown as T;
  }

  const farPlane: number = scalar(a.farPlane, b.farPlane);
  const fogDensity: number = scalar(a.fogDensity, b.fogDensity);
  const fogDistance: number = toFogDistance(engine, scalar(a.fogDistance, b.fogDistance), farPlane);

  let sunColor: TRendererVector = vector(a.sunColor, b.sunColor);
  let sunDirection: TRendererVector;

  switch (sun.kind) {
    case EWeatherSun.AUTHORED:
      sunDirection = normalise(vector(a.sunDirection ?? DOWN, b.sunDirection ?? DOWN));
      break;

    // The engine passes the mixed `exec_time`, which runs backwards across midnight; the time of day it stands for is
    // the same everywhere else.
    case EWeatherSun.DYNAMIC: {
      const dynamic: IDynamicSun = toDynamicSun(time, scalar(a.sunAzimuth, b.sunAzimuth));

      sunColor = [sunColor[0] * dynamic.blend, sunColor[1] * dynamic.blend, sunColor[2] * dynamic.blend];
      sunDirection = dynamic.direction;
      break;
    }

    case EWeatherSun.TABLE:
      sunDirection = toSunTableDirection(sun.positions, time);
      break;
  }

  return {
    ambientColor: vector(a.ambientColor, b.ambientColor),
    farPlane,
    fogColor: vector(a.fogColor, b.fogColor),
    fogDensity,
    fogDistance,
    fogFar: 0.99 * fogDistance,
    fogNear: (1 - fogDensity) * 0.85 * fogDistance,
    hemiColor: vector(a.hemiColor, b.hemiColor),
    keyframes: selected,
    skyColor: vector(a.skyColor, b.skyColor),
    skyRotation: scalar(a.skyRotation, b.skyRotation),
    sunColor,
    sunDirection,
    time,
    treeAmplitude: scalar(a.treeAmplitude, b.treeAmplitude),
    treeRotation: scalar(a.treeRotation, b.treeRotation),
    treeSpeed: scalar(a.treeSpeed, b.treeSpeed),
    treeWave: vector(a.treeWave, b.treeWave),
    waterIntensity: scalar(a.waterIntensity, b.waterIntensity),
    weight: f,
  };
}

/**
 * `SelectEnvs` on a forced start.
 *
 * @param keyframes - Sorted by time.
 * @param time - Seconds since midnight.
 * @returns The first keyframe at or after the time and the one before it, the last and the first around midnight;
 *   null for none.
 */
export function selectWeatherKeyframes(
  keyframes: ReadonlyArray<IRendererWeatherKeyframe>,
  time: number
): Nullable<readonly [number, number]> {
  const last: number = keyframes.length - 1;

  if (last < 0) {
    return null;
  }

  const next: number = keyframes.findIndex((keyframe: IRendererWeatherKeyframe) => keyframe.time >= time);

  return next <= 0 ? [last, 0] : [next - 1, next];
}

/**
 * `TimeWeight`.
 *
 * @param time - Seconds since midnight.
 * @param span - The two keyframes' times.
 * @returns How far the time is from the first to the second, around midnight where the first is later.
 */
export function weighWeatherTime(time: number, span: readonly [number, number]): number {
  const [from, to] = span;
  const length: number = toElapsed(from, to);

  if (Math.abs(length) < EPS) {
    return 0;
  }

  const isWithin: boolean = from > to ? time >= from || time <= to : time >= from && time <= to;

  return isWithin ? Math.min(Math.max(toElapsed(from, time) / length, 0), 1) : 0;
}

/** `TimeDiff`: seconds from one time of day to the next, around midnight where it comes first. */
function toElapsed(from: number, to: number): number {
  return from > to ? WEATHER_DAY_LENGTH - from + to : to - from;
}

/** Monolith's `clamp(fog_distance, 1.f, far_plane - 10)`, the low bound first. */
function toFogDistance(engine: ERendererWeatherEngine, distance: number, farPlane: number): number {
  if (engine === ERendererWeatherEngine.VANILLA) {
    return distance;
  }

  return distance < 1 ? 1 : Math.min(distance, farPlane - 10);
}

function normalise([x, y, z]: TRendererVector): TRendererVector {
  const length: number = Math.hypot(x, y, z);

  return length > 0 ? [x / length, y / length, z / length] : DOWN;
}
