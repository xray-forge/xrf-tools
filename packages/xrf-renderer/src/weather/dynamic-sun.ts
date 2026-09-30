import { clamp, EPS_S, saturate, toDirection, toRadians } from "@xrf/math";

import { TRendererVector } from "#/contract/renderer-vector";
import { WEATHER_DAY_LENGTH } from "#/weather/weather-day";

/** Where the engine puts its sun: Chernobyl's latitude, and a longitude its clock is offset by. */
const LATITUDE: number = toRadians(50.27);
const LONGITUDE: number = -30.4;

/** The elevations the sun's colour fades out between as it sets. */
const LOWEST: number = toRadians(1);
const FULL: number = toRadians(3);

/**
 * OpenXRay's astronomical sun at one time of day.
 */
export interface IDynamicSun {
  /** The way sunlight travels, in engine space. */
  direction: TRendererVector;
  /** How much of the sun's colour is left as it sets. */
  blend: number;
}

/**
 * `calculate_dynamic_sun_dir`.
 *
 * @param time - Seconds since midnight.
 * @param azimuth - Radians the sun is turned by.
 * @returns Where the sun stands, and how much of it shines.
 */
export function toDynamicSun(time: number, azimuth: number): IDynamicSun {
  const g: number = toRadians((360 / 365.25) * (180 + time / WEATHER_DAY_LENGTH));
  const declination: number = toRadians(
    0.396372 -
      22.91327 * Math.cos(g) +
      4.02543 * Math.sin(g) -
      0.387205 * Math.cos(2 * g) +
      0.051967 * Math.sin(2 * g) -
      0.154527 * Math.cos(3 * g) +
      0.084798 * Math.sin(3 * g)
  );
  const correction: number =
    0.004297 +
    0.107029 * Math.cos(g) -
    1.837877 * Math.sin(g) -
    0.837378 * Math.cos(2 * g) -
    2.340475 * Math.sin(2 * g);

  let hourAngle: number = (time / (WEATHER_DAY_LENGTH / 24) - 12) * 15 + LONGITUDE + correction;

  if (hourAngle > 180) {
    hourAngle -= 360;
  }

  if (hourAngle < -180) {
    hourAngle += 360;
  }

  const zenith: number = Math.acos(
    clamp(
      Math.sin(LATITUDE) * Math.sin(declination) +
        Math.cos(LATITUDE) * Math.cos(declination) * Math.cos(toRadians(hourAngle)),
      -1,
      1
    )
  );
  const across: number = Math.sin(zenith) * Math.cos(LATITUDE);
  const azimuthCos: number =
    Math.abs(across) < EPS_S
      ? 0
      : clamp((Math.sin(declination) - Math.sin(LATITUDE) * Math.cos(zenith)) / across, -1, 1);
  const elevation: number = Math.max(Math.PI / 2 - zenith, LOWEST);
  const heading: number = Math.acos(azimuthCos) + azimuth;

  return {
    blend: saturate((elevation - LOWEST) / (FULL - LOWEST)),
    direction: toDirection({ heading: hourAngle < 0 ? 2 * Math.PI - heading : heading, pitch: -elevation }),
  };
}
