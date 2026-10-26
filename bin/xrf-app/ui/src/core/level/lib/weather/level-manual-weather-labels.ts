import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_KEYS } from "@/core/level/lib/weather/level-manual-weather-keys";

/** What each field of a keyframe set by hand is called on screen; its weather config key rides in its tooltip. */
export const LEVEL_MANUAL_WEATHER_LABELS: Readonly<Record<keyof ILevelManualWeather, string>> = {
  ambientColor: "Ambient colour",
  cloudsColor: "Cloud colour",
  cloudsRotation: "Cloud rotation",
  cloudsTexture: "Cloud texture",
  farPlane: "View distance",
  fogColor: "Fog colour",
  fogDensity: "Fog density",
  fogDistance: "Fog distance",
  hemisphereColor: "Hemisphere colour",
  rainColor: "Rain colour",
  rainDensity: "Rain density",
  skyColor: "Sky colour",
  skyRotation: "Sky rotation",
  skyTexture: "Sky texture",
  sun: "Flare set",
  // The engine stands the sun by `setHP(sun_altitude, sun_longitude)`: the first is its bearing, the second its pitch.
  sunAltitude: "Bearing",
  sunColor: "Sun colour",
  sunLongitude: "Elevation",
  sunShaftsIntensity: "Sun shaft intensity",
  thunderboltCollection: "Lightning bolts",
  thunderboltDuration: "Bolt duration",
  thunderboltPeriod: "Time between bolts",
  treesAmplitude: "Tree sway",
  treesRotation: "Tree sway angle",
  treesSpeed: "Tree sway speed",
  treesWave: "Tree wave",
  waterIntensity: "Water intensity",
  windDirection: "Wind direction",
  windVelocity: "Wind speed",
};

/** The fields shown negated: `sun_longitude` points the sun down as it grows, so its elevation is its negation. */
export const LEVEL_MANUAL_WEATHER_NEGATED: ReadonlySet<keyof ILevelManualWeather> = new Set(["sunLongitude"]);

/**
 * @param field - A field of the keyframe set by hand.
 * @returns Its tooltip, naming the key a weather config writes it under.
 */
export function describeLevelManualWeatherKey(field: keyof ILevelManualWeather): string {
  return `Weather key: ${LEVEL_MANUAL_WEATHER_KEYS[field]}`;
}
