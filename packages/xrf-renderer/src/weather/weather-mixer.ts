import { Nullable } from "@xrf/types";

import { TRendererVector } from "#/contract/renderer-vector";
import { ERendererWeatherEngine } from "#/contract/weather/renderer-weather-engine";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IDynamicSun, toDynamicSun } from "#/weather/dynamic-sun";
import { toSunTableDirection } from "#/weather/sun-table-direction";
import { IWeatherCycleMix } from "#/weather/weather-cycle-mix";
import { toWeatherTimeOfDay, WEATHER_DAY_LENGTH } from "#/weather/weather-day";
import { IWeatherMix } from "#/weather/weather-mix";
import { IWeatherMixPoint } from "#/weather/weather-mix-point";
import { IWeatherModifiersSum, toWeatherModifiersSum, WEATHER_MODIFIER_FLAGS } from "#/weather/weather-modifiers-sum";
import { IWeatherPairMixer } from "#/weather/weather-pair-mixer";
import { EWeatherSun } from "#/weather/weather-sun";

/** `EPS`, what `TimeWeight` takes a zero span by. */
const EPS: number = 0.00001;

/** Where an authored sun stands for a keyframe that stands none. */
const DOWN: TRendererVector = [0, -1, 0];

/**
 * What a cycle is mixed from: `CEnvironment::lerp`, with the modifiers reaching the view.
 */
export interface IWeatherMixer extends Omit<IWeatherPairMixer, "pair"> {
  /** Sorted by time. */
  keyframes: ReadonlyArray<IRendererWeatherKeyframe>;
}

/**
 * The keyframes around a time of day, selected as `SelectEnvs` selects them on a forced start, then mixed.
 *
 * @param mixer - What is mixed.
 * @param point - When, wrapped into the day, and from where.
 * @returns The mix and which keyframes it is of, or null for a cycle without keyframes.
 */
export function mixWeather(mixer: IWeatherMixer, point: IWeatherMixPoint): Nullable<IWeatherCycleMix> {
  const { keyframes } = mixer;
  const selected: Nullable<readonly [number, number]> = selectWeatherKeyframes(
    keyframes,
    toWeatherTimeOfDay(point.time)
  );

  return selected
    ? {
        ...mixWeatherPair({ ...mixer, pair: [keyframes[selected[0]], keyframes[selected[1]]] }, point),
        keyframes: selected,
      }
    : null;
}

/**
 * `CEnvDescriptorMixer::lerp` of two keyframes at a time of day, seen from a point.
 *
 * @param mixer - The pair, and what it is mixed by.
 * @param point - When, wrapped into the day, and from where.
 * @returns The mix.
 */
export function mixWeatherPair(mixer: IWeatherPairMixer, point: IWeatherMixPoint): IWeatherMix {
  const { pair, engine, sun, modifiers } = mixer;
  const [a, b] = pair;
  const time: number = toWeatherTimeOfDay(point.time);
  const modified: IWeatherModifiersSum = toWeatherModifiersSum(modifiers, point.view);
  const scale: number = 1 / (modified.power + 1);
  const f: number = weighWeatherTime(time, [a.time, b.time]);

  function scalar(from: number, to: number): number {
    return (1 - f) * from + f * to;
  }

  function vector<T extends ReadonlyArray<number>>(from: T, to: T): T {
    return from.map((value: number, index: number) => scalar(value, to[index])) as unknown as T;
  }

  // A value some modifier reaches is the mix plus what they add, of which the environment keeps its share.
  function modify(flag: number, value: number, added: number): number {
    return modified.flags & flag ? (value + added) * scale : value;
  }

  function modifyVector(flag: number, value: TRendererVector, added: TRendererVector): TRendererVector {
    return [modify(flag, value[0], added[0]), modify(flag, value[1], added[1]), modify(flag, value[2], added[2])];
  }

  const farPlane: number = modify(WEATHER_MODIFIER_FLAGS.FAR_PLANE, scalar(a.farPlane, b.farPlane), modified.farPlane);
  const fogDensity: number = modify(
    WEATHER_MODIFIER_FLAGS.FOG_DENSITY,
    scalar(a.fogDensity, b.fogDensity),
    modified.fogDensity
  );
  const hemiColor: TRendererVector = modifyVector(
    WEATHER_MODIFIER_FLAGS.HEMI_COLOR,
    [
      scalar(a.hemiColor[0], b.hemiColor[0]),
      scalar(a.hemiColor[1], b.hemiColor[1]),
      scalar(a.hemiColor[2], b.hemiColor[2]),
    ],
    modified.hemiColor
  );
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
    ambientColor: modifyVector(
      WEATHER_MODIFIER_FLAGS.AMBIENT_COLOR,
      vector(a.ambientColor, b.ambientColor),
      modified.ambient
    ),
    cloudsColor: vector(a.cloudsColor, b.cloudsColor),
    cloudsRotation: scalar(a.cloudsRotation, b.cloudsRotation),
    farPlane,
    fogColor: modifyVector(WEATHER_MODIFIER_FLAGS.FOG_COLOR, vector(a.fogColor, b.fogColor), modified.fogColor),
    fogDensity,
    fogDistance,
    fogFar: 0.99 * fogDistance,
    fogNear: (1 - fogDensity) * 0.85 * fogDistance,
    hemiColor: [...hemiColor, scalar(a.hemiColor[3], b.hemiColor[3])],
    modifiers: modified.count,
    skyColor: modifyVector(WEATHER_MODIFIER_FLAGS.SKY_COLOR, vector(a.skyColor, b.skyColor), modified.skyColor),
    skyRotation: scalar(a.skyRotation, b.skyRotation),
    sunColor,
    sunDirection,
    time,
    view: point.view,
    treeAmplitude: scalar(a.treeAmplitude, b.treeAmplitude),
    treeRotation: scalar(a.treeRotation, b.treeRotation),
    treeSpeed: scalar(a.treeSpeed, b.treeSpeed),
    treeWave: vector(a.treeWave, b.treeWave),
    waterIntensity: scalar(a.waterIntensity, b.waterIntensity),
    thunderboltCollection: f < 0.5 ? a.thunderboltCollection : b.thunderboltCollection,
    thunderboltDuration: scalar(a.thunderboltDuration, b.thunderboltDuration),
    thunderboltPeriod: scalar(a.thunderboltPeriod, b.thunderboltPeriod),
    rainColor: vector(a.rainColor, b.rainColor),
    rainDensity: scalar(a.rainDensity, b.rainDensity),
    windDirection: scalar(a.windDirection, b.windDirection),
    windVelocity: scalar(a.windVelocity, b.windVelocity),
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
