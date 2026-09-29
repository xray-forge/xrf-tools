import { EXrayEngine, XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { formatLevelWeatherTime } from "@/core/level/lib/weather/level-weather-time";

/** Decimals a number is written with, as the game's own weathers write theirs. */
const DIGITS: number = 6;

/** What a keyframe set by hand is written as. */
export interface ILevelManualWeatherLtxInput {
  manual: ILevelManualWeather;
  /** Seconds since midnight, which names the section. */
  time: number;
  /** The engine whose keys are written: Monolith stands its sun by the sun table and reads no tree keys but one. */
  engine: XrayEngine;
}

/**
 * A keyframe set by hand as a weather config's `[HH:MM:SS]` section, with the keys the engine target reads; `sun`,
 * `ambient` and the thunderbolts are the file's own and are left out.
 *
 * @param input - The keyframe, its time and the engine target.
 * @returns The section's text.
 */
export function toLevelManualWeatherLtx(input: ILevelManualWeatherLtxInput): string {
  const { manual, time, engine } = input;
  const isVanilla: boolean = engine === EXrayEngine.VANILLA;
  const [cloudsRed, cloudsGreen, cloudsBlue, cover] = manual.cloudsColor;
  const keys: Array<[string, string]> = [
    ["sky_texture", manual.skyTexture],
    ["sky_color", toList(manual.skyColor)],
    ["sky_rotation", toNumber(manual.skyRotation)],
    ["clouds_texture", manual.cloudsTexture],
    // The fifth component scales the colour by half of it, so two keeps it as held.
    ["clouds_color", toList([cloudsRed, cloudsGreen, cloudsBlue, cover, 2])],
    ...(isVanilla ? [["clouds_rotation", toNumber(manual.cloudsRotation)] as [string, string]] : []),
    ["far_plane", toNumber(manual.farPlane)],
    ["fog_color", toList(manual.fogColor)],
    ["fog_distance", toNumber(manual.fogDistance)],
    ["fog_density", toNumber(manual.fogDensity)],
    ["rain_density", toNumber(manual.rainDensity)],
    ["rain_color", toList(manual.rainColor)],
    ["wind_velocity", toNumber(manual.windVelocity)],
    ["wind_direction", toNumber(manual.windDirection)],
    ["hemisphere_color", toList(manual.hemisphereColor)],
    ["sun_color", toList(manual.sunColor)],
    ["ambient_color", toList(manual.ambientColor)],
    ["water_intensity", toNumber(manual.waterIntensity)],
    ...(isVanilla
      ? ([
          ["sun_altitude", toNumber(manual.sunAltitude)],
          ["sun_longitude", toNumber(manual.sunLongitude)],
          ["trees_amplitude", toNumber(manual.treesAmplitude)],
          ["trees_speed", toNumber(manual.treesSpeed)],
          ["trees_rotation", toNumber(manual.treesRotation)],
          ["trees_wave", toList(manual.treesWave)],
        ] as Array<[string, string]>)
      : ([["tree_amplitude_intensity", toNumber(manual.treesAmplitude)]] as Array<[string, string]>)),
  ];
  const width: number = Math.max(...keys.map(([key]) => key.length));

  return [
    `[${formatLevelWeatherTime(time, true)}]`,
    ...keys.map(([key, value]) => `${key.padEnd(width)} = ${value}`),
    "",
  ].join("\n");
}

function toNumber(value: number): string {
  return String(Number(value.toFixed(DIGITS)));
}

function toList(values: ReadonlyArray<number>): string {
  return values.map(toNumber).join(", ");
}
