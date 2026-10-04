import { saturate, toDegrees, toDirection, toHeadingPitch, toRadians } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import { Vector3d } from "@/core/ipc/types/xrf-math";
import { TRenderVector } from "@/core/render/lib/scene/render-space";

/** What names a sky's irradiance cube after the sky's own reference. */
const ENVIRONMENT_SUFFIX: string = "#small";

/**
 * A keyframe set by hand, named by its keys as a weather config writes them and holding what it would write: angles
 * in degrees, colours as the engine holds them.
 */
export interface ILevelManualWeather {
  skyTexture: string;
  skyColor: TRenderVector;
  /** Degrees. */
  skyRotation: number;
  /** Empty for none. */
  cloudsTexture: string;
  /** The colour as the engine holds it, scaled by its multiplier, and the cover in alpha. */
  cloudsColor: readonly [number, number, number, number];
  /** Degrees. */
  cloudsRotation: number;
  farPlane: number;
  fogColor: TRenderVector;
  fogDistance: number;
  fogDensity: number;
  hemisphereColor: readonly [number, number, number, number];
  sunColor: TRenderVector;
  /** Degrees; `setHP` turns the heading by it. */
  sunAltitude: number;
  /** Degrees; `setHP` tilts the pitch by it. */
  sunLongitude: number;
  /** The lens flare drawn, a `suns.ltx` section: the sun's sprite, its flares and gradient; empty for none. */
  sun: string;
  /** How dense the light shafts through the sun's shadow are. */
  sunShaftsIntensity: number;
  ambientColor: TRenderVector;
  waterIntensity: number;
  rainDensity: number;
  rainColor: TRenderVector;
  windVelocity: number;
  /** Degrees. */
  windDirection: number;
  treesAmplitude: number;
  treesSpeed: number;
  treesRotation: number;
  treesWave: TRenderVector;
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
  sun: "gradient1",
  sunShaftsIntensity: 0,
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

/** What a keyframe set by hand reads as for every key it does not set: the engine's defaults where it reads them. */
const MANUAL_DESCRIPTOR_REST: Pick<
  WeatherDescriptor,
  | "ambient"
  | "isSunFixed"
  | "sunAzimuth"
  | "hemiVibrance"
  | "hemiContrast"
  | "wetSurfaceFactor"
  | "volumetricIntensityFactor"
  | "volumetricDistanceFactor"
  | "bloomThreshold"
  | "bloomExposure"
  | "bloomSkyIntensity"
> = {
  ambient: null,
  bloomExposure: 3,
  bloomSkyIntensity: 0.6,
  bloomThreshold: 3.5,
  hemiContrast: 1,
  hemiVibrance: 1,
  isSunFixed: false,
  sunAzimuth: 0,
  volumetricDistanceFactor: 1,
  volumetricIntensityFactor: 1,
  wetSurfaceFactor: 0,
};

/**
 * @param manual - A keyframe set by hand.
 * @param time - Seconds since midnight it stands at.
 * @returns It as the engine holds a loaded keyframe: radians, the sun's direction built by `setHP`.
 */
export function toLevelManualDescriptor(manual: ILevelManualWeather, time: number): WeatherDescriptor {
  return {
    ...MANUAL_DESCRIPTOR_REST,
    ambientColor: [...manual.ambientColor],
    cloudsColor: [...manual.cloudsColor],
    cloudsRotation: toRadians(manual.cloudsRotation),
    cloudsTexture: manual.cloudsTexture,
    farPlane: manual.farPlane,
    fogColor: [...manual.fogColor],
    fogDensity: manual.fogDensity,
    fogDistance: manual.fogDistance,
    hemiColor: [...manual.hemisphereColor],
    rainColor: [...manual.rainColor],
    rainDensity: saturate(manual.rainDensity),
    skyColor: [...manual.skyColor],
    skyRotation: toRadians(manual.skyRotation),
    skyTexture: manual.skyTexture,
    skyTextureEnv: `${manual.skyTexture}${ENVIRONMENT_SUFFIX}`,
    sunColor: [...manual.sunColor],
    sunDirection: toDirection({ heading: toRadians(manual.sunAltitude), pitch: toRadians(manual.sunLongitude) }),
    time: Math.round(time),
    treeAmplitude: manual.treesAmplitude,
    treeRotation: manual.treesRotation,
    treeSpeed: manual.treesSpeed,
    treeWave: [...manual.treesWave],
    // Zero without a collection, as the engine loads a keyframe that strikes with none.
    thunderboltCollection: manual.thunderboltCollection || null,
    thunderboltDuration: manual.thunderboltCollection ? manual.thunderboltDuration : 0,
    thunderboltPeriod: manual.thunderboltCollection ? manual.thunderboltPeriod : 0,
    waterIntensity: manual.waterIntensity,
    sun: manual.sun || null,
    sunShaftsIntensity: manual.sunShaftsIntensity,
    windDirection: toRadians(manual.windDirection),
    windVelocity: manual.windVelocity,
  };
}

/**
 * A keyframe set by hand from what the weather shows: the angles `getHP` reads back out of the sun's direction, so
 * however the weather stood it, the keyframe stands it the same.
 *
 * @param current - What the renderer mixes now, as one keyframe.
 * @returns The same as a keyframe set by hand, a number the renderer could not write as zero.
 */
export function toLevelManualWeather(current: WeatherDescriptor): ILevelManualWeather {
  return {
    ...toSunAngles(current.sunDirection ? toTriple(current.sunDirection) : [0, -1, 0]),
    ambientColor: toTriple(current.ambientColor),
    cloudsColor: [
      current.cloudsColor[0] ?? 0,
      current.cloudsColor[1] ?? 0,
      current.cloudsColor[2] ?? 0,
      current.cloudsColor[3] ?? 0,
    ],
    cloudsRotation: toDegrees(current.cloudsRotation ?? 0),
    cloudsTexture: current.cloudsTexture,
    farPlane: current.farPlane ?? 0,
    fogColor: toTriple(current.fogColor),
    fogDensity: current.fogDensity ?? 0,
    fogDistance: current.fogDistance ?? 0,
    hemisphereColor: [
      current.hemiColor[0] ?? 0,
      current.hemiColor[1] ?? 0,
      current.hemiColor[2] ?? 0,
      current.hemiColor[3] ?? 0,
    ],
    rainColor: toTriple(current.rainColor),
    rainDensity: current.rainDensity ?? 0,
    skyColor: toTriple(current.skyColor),
    skyRotation: toDegrees(current.skyRotation ?? 0),
    skyTexture: current.skyTexture,
    sunColor: toTriple(current.sunColor),
    treesAmplitude: current.treeAmplitude ?? 0,
    treesRotation: current.treeRotation ?? 0,
    treesSpeed: current.treeSpeed ?? 0,
    treesWave: toTriple(current.treeWave),
    thunderboltCollection: current.thunderboltCollection ?? "",
    thunderboltDuration: current.thunderboltDuration ?? 0,
    thunderboltPeriod: current.thunderboltPeriod ?? 0,
    waterIntensity: current.waterIntensity ?? 0,
    sun: current.sun ?? "",
    sunShaftsIntensity: current.sunShaftsIntensity ?? 0,
    windDirection: toDegrees(current.windDirection ?? 0),
    windVelocity: current.windVelocity ?? 0,
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

/** `getHP` of the sun's direction, in degrees: `sun_altitude` is its heading and `sun_longitude` its pitch. */
function toSunAngles(direction: TRenderVector): Pick<ILevelManualWeather, "sunAltitude" | "sunLongitude"> {
  const { heading, pitch } = toHeadingPitch(direction);

  return { sunAltitude: toDegrees(heading), sunLongitude: toDegrees(pitch) };
}

function toTriple(value: readonly [Nullable<number>, Nullable<number>, Nullable<number>]): TRenderVector {
  return [value[0] ?? 0, value[1] ?? 0, value[2] ?? 0];
}

function isFiniteNumber(value: unknown): boolean {
  return typeof value === "number" && Number.isFinite(value);
}
