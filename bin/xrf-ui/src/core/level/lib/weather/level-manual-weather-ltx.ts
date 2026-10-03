import { EXrayEngine, XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_KEYS } from "@/core/level/lib/weather/level-manual-weather-keys";
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
 * A keyframe set by hand as a weather config's `[HH:MM:SS]` section, with the keys the engine target reads; `sun` and
 * `ambient` are the file's own and are left out.
 *
 * @param input - The keyframe, its time and the engine target.
 * @returns The section's text.
 */
export function toLevelManualWeatherLtx(input: ILevelManualWeatherLtxInput): string {
  const { manual, time, engine } = input;
  const isVanilla: boolean = engine === EXrayEngine.VANILLA;
  const [cloudsRed, cloudsGreen, cloudsBlue, cover] = manual.cloudsColor;
  const keys: Array<[string, string]> = [
    [LEVEL_MANUAL_WEATHER_KEYS.skyTexture, manual.skyTexture],
    [LEVEL_MANUAL_WEATHER_KEYS.skyColor, toList(manual.skyColor)],
    [LEVEL_MANUAL_WEATHER_KEYS.skyRotation, toNumber(manual.skyRotation)],
    [LEVEL_MANUAL_WEATHER_KEYS.cloudsTexture, manual.cloudsTexture],
    // The fifth component scales the colour by half of it, so two keeps it as held.
    [LEVEL_MANUAL_WEATHER_KEYS.cloudsColor, toList([cloudsRed, cloudsGreen, cloudsBlue, cover, 2])],
    ...(isVanilla
      ? [[LEVEL_MANUAL_WEATHER_KEYS.cloudsRotation, toNumber(manual.cloudsRotation)] as [string, string]]
      : []),
    [LEVEL_MANUAL_WEATHER_KEYS.farPlane, toNumber(manual.farPlane)],
    [LEVEL_MANUAL_WEATHER_KEYS.fogColor, toList(manual.fogColor)],
    [LEVEL_MANUAL_WEATHER_KEYS.fogDistance, toNumber(manual.fogDistance)],
    [LEVEL_MANUAL_WEATHER_KEYS.fogDensity, toNumber(manual.fogDensity)],
    [LEVEL_MANUAL_WEATHER_KEYS.rainDensity, toNumber(manual.rainDensity)],
    [LEVEL_MANUAL_WEATHER_KEYS.rainColor, toList(manual.rainColor)],
    [LEVEL_MANUAL_WEATHER_KEYS.windVelocity, toNumber(manual.windVelocity)],
    [LEVEL_MANUAL_WEATHER_KEYS.windDirection, toNumber(manual.windDirection)],
    [LEVEL_MANUAL_WEATHER_KEYS.hemisphereColor, toList(manual.hemisphereColor)],
    [LEVEL_MANUAL_WEATHER_KEYS.sunColor, toList(manual.sunColor)],
    [LEVEL_MANUAL_WEATHER_KEYS.sun, manual.sun],
    [LEVEL_MANUAL_WEATHER_KEYS.sunShaftsIntensity, toNumber(manual.sunShaftsIntensity)],
    [LEVEL_MANUAL_WEATHER_KEYS.ambientColor, toList(manual.ambientColor)],
    [LEVEL_MANUAL_WEATHER_KEYS.waterIntensity, toNumber(manual.waterIntensity)],
    [LEVEL_MANUAL_WEATHER_KEYS.thunderboltCollection, manual.thunderboltCollection],
    [LEVEL_MANUAL_WEATHER_KEYS.thunderboltDuration, toNumber(manual.thunderboltDuration)],
    [LEVEL_MANUAL_WEATHER_KEYS.thunderboltPeriod, toNumber(manual.thunderboltPeriod)],
    ...(isVanilla
      ? ([
          [LEVEL_MANUAL_WEATHER_KEYS.sunAltitude, toNumber(manual.sunAltitude)],
          [LEVEL_MANUAL_WEATHER_KEYS.sunLongitude, toNumber(manual.sunLongitude)],
          [LEVEL_MANUAL_WEATHER_KEYS.treesAmplitude, toNumber(manual.treesAmplitude)],
          [LEVEL_MANUAL_WEATHER_KEYS.treesSpeed, toNumber(manual.treesSpeed)],
          [LEVEL_MANUAL_WEATHER_KEYS.treesRotation, toNumber(manual.treesRotation)],
          [LEVEL_MANUAL_WEATHER_KEYS.treesWave, toList(manual.treesWave)],
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
