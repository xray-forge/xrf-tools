import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";

/** The sun's and the light's keys of a keyframe set by hand, set back together. */
export const LEVEL_MANUAL_WEATHER_SUN_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "sunColor",
  "sunAltitude",
  "sunLongitude",
  "ambientColor",
  "hemisphereColor",
  "sun",
  "sunShaftsIntensity",
];

/** The sky's keys of a keyframe set by hand, set back together. */
export const LEVEL_MANUAL_WEATHER_SKY_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "skyTexture",
  "skyColor",
  "skyRotation",
];

/** The clouds' keys of a keyframe set by hand, set back together. */
export const LEVEL_MANUAL_WEATHER_CLOUDS_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "cloudsTexture",
  "cloudsColor",
  "cloudsRotation",
];

/** The fog's and the view distance's keys of a keyframe set by hand, set back together. */
export const LEVEL_MANUAL_WEATHER_FOG_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "fogColor",
  "fogDistance",
  "fogDensity",
  "farPlane",
];

/** The rain's keys of a keyframe set by hand, set back together. */
export const LEVEL_MANUAL_WEATHER_RAIN_KEYS: ReadonlyArray<keyof ILevelManualWeather> = ["rainDensity", "rainColor"];

/** The thunder's keys of a keyframe set by hand, set back together. */
export const LEVEL_MANUAL_WEATHER_THUNDER_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "thunderboltCollection",
  "thunderboltPeriod",
  "thunderboltDuration",
];

/** The water's key of a keyframe set by hand, set back on its own. */
export const LEVEL_MANUAL_WEATHER_WATER_KEYS: ReadonlyArray<keyof ILevelManualWeather> = ["waterIntensity"];

/** The wind's and the trees' keys of a keyframe set by hand, set back together. */
export const LEVEL_MANUAL_WEATHER_WIND_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "windVelocity",
  "windDirection",
  "treesAmplitude",
  "treesSpeed",
  "treesRotation",
  "treesWave",
];
