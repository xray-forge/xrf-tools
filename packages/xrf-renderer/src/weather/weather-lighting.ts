import { EPS_L, toDegrees } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { DEFAULT_RENDERER_GRASS_WIND, DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { IWeatherLightingInput } from "#/weather/weather-lighting-input";
import { IWeatherMixedKeyframeInput } from "#/weather/weather-mixed-keyframe-input";
import { WeatherTextures } from "#/weather/weather-textures";

/**
 * A weather's mix as the scene is lit by it, in renderer space: the view's switches hiding what they hide, the skies
 * and clouds of both keyframes by their texture keys, and no bolt striking.
 *
 * @param input - The weather, its pair, their mix and the view's switches.
 * @returns What the scene is lit by.
 */
export function toWeatherLighting(input: IWeatherLightingInput): IRendererLighting {
  const { weather, pair, mix, control } = input;
  const { isClouded, isFogged, isRainy, isWindy } = control;
  const [a, b] = pair;
  const [x, y, z] = mix.sunDirection;

  return {
    ambientColor: mix.ambientColor,
    engine: weather.engine,
    fog: isFogged
      ? { color: mix.fogColor, density: mix.fogDensity, distance: mix.fogDistance, farPlane: mix.farPlane }
      : null,
    grass: isWindy ? DEFAULT_RENDERER_GRASS_WIND : null,
    hemisphereColor: [mix.hemiColor[0], mix.hemiColor[1], mix.hemiColor[2]],
    // Under `EPS_L` it does not rain at all.
    rain:
      isRainy && weather.rain && mix.rainDensity >= EPS_L
        ? {
            color: mix.rainColor,
            density: mix.rainDensity,
            windDirection: mix.windDirection,
            windVelocity: mix.windVelocity,
          }
        : null,
    sky: {
      blend: mix.weight,
      clouds: {
        color: mix.cloudsColor,
        rotation: toDegrees(mix.cloudsRotation),
        textures: isClouded ? [toCloudsKey(a), toCloudsKey(b)] : [null, null],
      },
      color: mix.skyColor,
      environments: [WeatherTextures.toKey(a.skyTextureEnv), WeatherTextures.toKey(b.skyTextureEnv)],
      rotation: toDegrees(mix.skyRotation),
      textures: [WeatherTextures.toKey(a.skyTexture), WeatherTextures.toKey(b.skyTexture)],
    },
    skyIrradiance: DEFAULT_RENDERER_LIGHTING.skyIrradiance,
    sunColor: mix.sunColor,
    // Engine `z` negated into renderer space.
    sunDirection: [x, y, -z],
    thunderbolt: null,
    trees: isWindy
      ? { amplitude: mix.treeAmplitude, rotation: mix.treeRotation, speed: mix.treeSpeed, wave: mix.treeWave }
      : null,
    waterIntensity: mix.waterIntensity,
  };
}

/**
 * A mix as one keyframe, which a keyframe set by hand is seeded from: the heavier keyframe's textures, the sun where it
 * stands.
 *
 * @param input - The pair and their mix, and the time the keyframe stands at.
 * @returns The keyframe.
 */
export function toWeatherMixedKeyframe(input: IWeatherMixedKeyframeInput): IRendererWeatherKeyframe {
  const { pair, mix, time } = input;
  const heavier: IRendererWeatherKeyframe = pair[mix.weight >= 0.5 ? 1 : 0];

  return {
    ambientColor: mix.ambientColor,
    cloudsColor: mix.cloudsColor,
    cloudsRotation: mix.cloudsRotation,
    cloudsTexture: heavier.cloudsTexture,
    farPlane: mix.farPlane,
    fogColor: mix.fogColor,
    fogDensity: mix.fogDensity,
    fogDistance: mix.fogDistance,
    hemiColor: mix.hemiColor,
    rainColor: mix.rainColor,
    rainDensity: mix.rainDensity,
    skyColor: mix.skyColor,
    skyRotation: mix.skyRotation,
    skyTexture: heavier.skyTexture,
    skyTextureEnv: heavier.skyTextureEnv,
    sunAzimuth: heavier.sunAzimuth,
    sunColor: mix.sunColor,
    sunDirection: mix.sunDirection,
    thunderboltCollection: mix.thunderboltCollection,
    thunderboltDuration: mix.thunderboltDuration,
    thunderboltPeriod: mix.thunderboltPeriod,
    time,
    treeAmplitude: mix.treeAmplitude,
    treeRotation: mix.treeRotation,
    treeSpeed: mix.treeSpeed,
    treeWave: mix.treeWave,
    waterIntensity: mix.waterIntensity,
    windDirection: mix.windDirection,
    windVelocity: mix.windVelocity,
  };
}

/** The key a keyframe's clouds are put under, or null for a keyframe that names none. */
function toCloudsKey(keyframe: IRendererWeatherKeyframe): Nullable<string> {
  return keyframe.cloudsTexture ? WeatherTextures.toKey(keyframe.cloudsTexture) : null;
}
