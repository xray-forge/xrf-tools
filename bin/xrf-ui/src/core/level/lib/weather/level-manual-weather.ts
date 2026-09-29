import { toDegrees, toRadians } from "@xrf/math";
import { IRendererWeatherKeyframe, TRendererVector } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { Vector3d } from "@/core/ipc/types/xrf-math";

/** What names a sky's irradiance cube after the sky's own reference. */
const ENVIRONMENT_SUFFIX: string = "#small";

/**
 * A keyframe set by hand, named by its keys as a weather config writes them and holding what it would write: angles
 * in degrees, colours as the engine holds them.
 */
export interface ILevelManualWeather {
  skyTexture: string;
  skyColor: TRendererVector;
  /** Degrees. */
  skyRotation: number;
  /** Empty for none. */
  cloudsTexture: string;
  /** The colour as the engine holds it, scaled by its multiplier, and the cover in alpha. */
  cloudsColor: readonly [number, number, number, number];
  /** Degrees. */
  cloudsRotation: number;
  farPlane: number;
  fogColor: TRendererVector;
  fogDistance: number;
  fogDensity: number;
  hemisphereColor: readonly [number, number, number, number];
  sunColor: TRendererVector;
  /** Degrees; `setHP` turns the heading by it. */
  sunAltitude: number;
  /** Degrees; `setHP` tilts the pitch by it. */
  sunLongitude: number;
  ambientColor: TRendererVector;
  waterIntensity: number;
  rainDensity: number;
  rainColor: TRendererVector;
  windVelocity: number;
  /** Degrees. */
  windDirection: number;
  treesAmplitude: number;
  treesSpeed: number;
  treesRotation: number;
  treesWave: TRendererVector;
  /** The `thunderbolt_collections.ltx` section struck with, empty for none. */
  thunderboltCollection: string;
  /** Seconds between strikes. */
  thunderboltPeriod: number;
  /** Seconds a strike lasts. */
  thunderboltDuration: number;
}

/** `default_clear` at noon (`configs/environment/weathers/default_clear.ltx`, `[12:00:00]`), with the engine's sway. */
export const DEFAULT_LEVEL_MANUAL_WEATHER: ILevelManualWeather = {
  ambientColor: [0.02, 0.02, 0.02],
  cloudsColor: [0, 0, 0, 0],
  cloudsRotation: 0,
  cloudsTexture: "sky\\sky_oblaka",
  farPlane: 350,
  fogColor: [0.304609, 0.328138, 0.367354],
  fogDensity: 0.9,
  fogDistance: 350,
  hemisphereColor: [0.470588, 0.368627, 0.329412, 1],
  rainColor: [0.68, 0.64, 0.6],
  rainDensity: 0,
  skyColor: [0.851001, 0.851001, 0.851001],
  skyRotation: 0,
  skyTexture: "sky\\sky_7_cube",
  sunAltitude: -68.999985,
  sunColor: [0.905882, 0.839216, 0.694118],
  sunLongitude: -30,
  treesAmplitude: 0.005,
  treesRotation: 10,
  treesSpeed: 1,
  treesWave: [0.1, 0.01, 0.11],
  thunderboltCollection: "",
  thunderboltDuration: 0,
  thunderboltPeriod: 0,
  waterIntensity: 1,
  windDirection: 0,
  windVelocity: 0,
};

/**
 * @param manual - A keyframe set by hand.
 * @returns The references it draws with: its sky, the sky's irradiance cube, and its clouds where it names any.
 */
export function listLevelManualWeatherTextures(manual: ILevelManualWeather): Array<string> {
  return [manual.skyTexture, `${manual.skyTexture}${ENVIRONMENT_SUFFIX}`, manual.cloudsTexture].filter(Boolean);
}

/**
 * @param manual - A keyframe set by hand.
 * @param time - Seconds since midnight it stands at.
 * @returns It as the renderer mixes a keyframe: radians, the sun's direction built by `setHP`.
 */
export function toLevelManualKeyframe(manual: ILevelManualWeather, time: number): IRendererWeatherKeyframe {
  return {
    ambientColor: manual.ambientColor,
    cloudsColor: manual.cloudsColor,
    cloudsRotation: toRadians(manual.cloudsRotation),
    cloudsTexture: manual.cloudsTexture,
    farPlane: manual.farPlane,
    fogColor: manual.fogColor,
    fogDensity: manual.fogDensity,
    fogDistance: manual.fogDistance,
    hemiColor: manual.hemisphereColor,
    rainColor: manual.rainColor,
    rainDensity: Math.min(Math.max(manual.rainDensity, 0), 1),
    skyColor: manual.skyColor,
    skyRotation: toRadians(manual.skyRotation),
    skyTexture: manual.skyTexture,
    skyTextureEnv: `${manual.skyTexture}${ENVIRONMENT_SUFFIX}`,
    sunAzimuth: 0,
    sunColor: manual.sunColor,
    sunDirection: toSunDirection(toRadians(manual.sunAltitude), toRadians(manual.sunLongitude)),
    time,
    treeAmplitude: manual.treesAmplitude,
    treeRotation: manual.treesRotation,
    treeSpeed: manual.treesSpeed,
    treeWave: manual.treesWave,
    // Zero without a collection, as the engine loads a keyframe that strikes with none.
    thunderboltCollection: manual.thunderboltCollection || null,
    thunderboltDuration: manual.thunderboltCollection ? manual.thunderboltDuration : 0,
    thunderboltPeriod: manual.thunderboltCollection ? manual.thunderboltPeriod : 0,
    waterIntensity: manual.waterIntensity,
    windDirection: toRadians(manual.windDirection),
    windVelocity: manual.windVelocity,
  };
}

/**
 * A keyframe set by hand from what the weather shows: the angles `getHP` reads back out of the sun's direction, so
 * however the weather stood it, the keyframe stands it the same.
 *
 * @param current - What the renderer mixes now, as one keyframe.
 * @returns The same as a keyframe set by hand.
 */
export function toLevelManualWeather(current: IRendererWeatherKeyframe): ILevelManualWeather {
  return {
    ...toSunAngles(current.sunDirection ?? [0, -1, 0]),
    ambientColor: current.ambientColor,
    cloudsColor: current.cloudsColor,
    cloudsRotation: toDegrees(current.cloudsRotation),
    cloudsTexture: current.cloudsTexture,
    farPlane: current.farPlane,
    fogColor: current.fogColor,
    fogDensity: current.fogDensity,
    fogDistance: current.fogDistance,
    hemisphereColor: current.hemiColor,
    rainColor: current.rainColor,
    rainDensity: current.rainDensity,
    skyColor: current.skyColor,
    skyRotation: toDegrees(current.skyRotation),
    skyTexture: current.skyTexture,
    sunColor: current.sunColor,
    treesAmplitude: current.treeAmplitude,
    treesRotation: current.treeRotation,
    treesSpeed: current.treeSpeed,
    treesWave: current.treeWave,
    thunderboltCollection: current.thunderboltCollection ?? "",
    thunderboltDuration: current.thunderboltDuration,
    thunderboltPeriod: current.thunderboltPeriod,
    waterIntensity: current.waterIntensity,
    windDirection: toDegrees(current.windDirection),
    windVelocity: current.windVelocity,
  };
}

/**
 * @param stored - What storage handed back.
 * @returns The keyframe, or null where any field does not read as what the keyframe holds there.
 */
export function readLevelManualWeather(stored: unknown): Nullable<ILevelManualWeather> {
  if (typeof stored !== "object" || stored === null) {
    return null;
  }

  const record: Record<string, unknown> = stored as Record<string, unknown>;
  const read: Record<string, unknown> = {};

  for (const [key, fallback] of Object.entries(DEFAULT_LEVEL_MANUAL_WEATHER)) {
    const value: unknown = record[key];
    const isRead: boolean = Array.isArray(fallback)
      ? Array.isArray(value) && value.length === fallback.length && value.every(isFiniteNumber)
      : typeof fallback === "string"
        ? typeof value === "string"
        : isFiniteNumber(value);

    if (!isRead) {
      return null;
    }

    read[key] = value;
  }

  return read as unknown as ILevelManualWeather;
}

/**
 * The angles a keyframe stands a level's own sun by, for lighting it the way xrLC did.
 *
 * @param direction - The direction the level's sun light travels, in the level's own axes.
 * @returns `sun_altitude` and `sun_longitude` in degrees, or null for no direction.
 */
export function toLevelManualSun(
  direction: Nullable<Vector3d>
): Nullable<Pick<ILevelManualWeather, "sunAltitude" | "sunLongitude">> {
  const x: number = direction?.x ?? 0;
  const y: number = direction?.y ?? 0;
  const z: number = direction?.z ?? 0;
  const length: number = Math.hypot(x, y, z);

  if (!length) {
    return null;
  }

  return toSunAngles([x / length, y / length, z / length]);
}

/** `Fvector::getHP`: a direction's heading and pitch, in degrees. */
function toSunAngles(direction: TRendererVector): Pick<ILevelManualWeather, "sunAltitude" | "sunLongitude"> {
  const [x, y, z] = direction;

  return {
    sunAltitude: toDegrees(Math.atan2(-x, z)),
    sunLongitude: toDegrees(Math.asin(Math.min(Math.max(y, -1), 1))),
  };
}

/** `Fvector::setHP(h, p)`, in engine space. */
function toSunDirection(heading: number, pitch: number): TRendererVector {
  return [-Math.cos(pitch) * Math.sin(heading), Math.sin(pitch), Math.cos(pitch) * Math.cos(heading)];
}

function isFiniteNumber(value: unknown): boolean {
  return typeof value === "number" && Number.isFinite(value);
}
