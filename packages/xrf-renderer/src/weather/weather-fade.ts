import { Nullable } from "@xrf/types";

import { IRendererClouds } from "#/contract/renderer-clouds";
import { IRendererFog } from "#/contract/renderer-fog";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererRainfall } from "#/contract/renderer-rainfall";
import { IRendererSky } from "#/contract/renderer-sky";
import { IRendererTreeWind } from "#/contract/renderer-tree-wind";
import { TRendererVector } from "#/contract/renderer-vector";
import { IWeatherFadeStep } from "#/weather/weather-fade-step";

/** `EPS_L`, under which it does not rain at all. */
const RAIN_THRESHOLD: number = 0.001;

/** Two of what a sky slot holds, and how far from the first to the second the shader blends. */
interface IWeatherSlots<T> {
  pair: readonly [T, T];
  blend: number;
}

/**
 * What a fade shows at one step: every value blended from what was shown to what the weather shows now, and the skies
 * walked from one pair to the other in thirds, so the two slots the sky has never show a cube they did not show a
 * moment before.
 *
 * @param step - What was shown, what is shown now, and how far between.
 * @returns The lighting to draw.
 */
export function toFadedLighting(step: IWeatherFadeStep): IRendererLighting {
  const { from, to } = step;
  const t: number = Math.min(Math.max(step.progress, 0), 1);

  if (t >= 1) {
    return to;
  }

  function lerp(a: number, b: number): number {
    return a + (b - a) * t;
  }

  function triple(a: TRendererVector, b: TRendererVector): TRendererVector {
    return [lerp(a[0], b[0]), lerp(a[1], b[1]), lerp(a[2], b[2])];
  }

  return {
    ambientColor: triple(from.ambientColor, to.ambientColor),
    fog: fadeFog(from.fog, to.fog, { lerp, triple }),
    grass: to.grass,
    hemisphereColor: triple(from.hemisphereColor, to.hemisphereColor),
    rain: fadeRain(from.rain, to.rain, { lerp, triple }),
    sky: fadeSky(from.sky, to.sky, t),
    skyIrradiance: triple(from.skyIrradiance, to.skyIrradiance),
    sunColor: triple(from.sunColor, to.sunColor),
    sunDirection: normalise(triple(from.sunDirection, to.sunDirection)),
    trees: fadeTrees(from.trees, to.trees, { lerp, triple }),
    waterIntensity: lerp(from.waterIntensity, to.waterIntensity),
  };
}

/** The blends a fade step makes of numbers and triples. */
interface IWeatherBlends {
  lerp: (a: number, b: number) => number;
  triple: (a: TRendererVector, b: TRendererVector) => TRendererVector;
}

function fadeFog(
  from: Nullable<IRendererFog>,
  to: Nullable<IRendererFog>,
  blends: IWeatherBlends
): Nullable<IRendererFog> {
  if (!from || !to) {
    return to;
  }

  return {
    color: blends.triple(from.color, to.color),
    density: blends.lerp(from.density, to.density),
    distance: blends.lerp(from.distance, to.distance),
    farPlane: blends.lerp(from.farPlane, to.farPlane),
  };
}

function fadeTrees(
  from: Nullable<IRendererTreeWind>,
  to: Nullable<IRendererTreeWind>,
  blends: IWeatherBlends
): Nullable<IRendererTreeWind> {
  if (!from || !to) {
    return to;
  }

  return {
    amplitude: blends.lerp(from.amplitude, to.amplitude),
    rotation: blends.lerp(from.rotation, to.rotation),
    speed: blends.lerp(from.speed, to.speed),
    wave: blends.triple(from.wave, to.wave),
  };
}

/** Rain that starts or stops fades from or to nothing, as though it had been falling at no density. */
function fadeRain(
  from: Nullable<IRendererRainfall>,
  to: Nullable<IRendererRainfall>,
  blends: IWeatherBlends
): Nullable<IRendererRainfall> {
  const either: Nullable<IRendererRainfall> = to ?? from;

  if (!either) {
    return null;
  }

  const density: number = blends.lerp(from?.density ?? 0, to?.density ?? 0);

  if (density < RAIN_THRESHOLD) {
    return null;
  }

  return {
    color: blends.triple(from?.color ?? either.color, to?.color ?? either.color),
    density,
    windDirection: blends.lerp(from?.windDirection ?? either.windDirection, to?.windDirection ?? either.windDirection),
    windVelocity: blends.lerp(from?.windVelocity ?? either.windVelocity, to?.windVelocity ?? either.windVelocity),
  };
}

function fadeSky(from: IRendererSky, to: IRendererSky, t: number): IRendererSky {
  // One blend drives every slot in the shader, so every slot walks, or none does.
  const isWalked: boolean =
    !isSamePair(from.textures, to.textures) ||
    !isSamePair(from.environments, to.environments) ||
    !isSamePair(from.clouds?.textures ?? [null, null], to.clouds?.textures ?? [null, null]);
  const progress: IWeatherWalk = { isWalked, t };
  const skies: IWeatherSlots<string> = walk(
    { blend: from.blend, pair: from.textures },
    { blend: to.blend, pair: to.textures },
    progress
  );
  const environments: IWeatherSlots<string> = walk(
    { blend: from.blend, pair: from.environments },
    { blend: to.blend, pair: to.environments },
    progress
  );
  const clouds: Nullable<IRendererClouds> = fadeClouds(from, to, progress);

  return {
    blend: skies.blend,
    clouds,
    color: [
      from.color[0] + (to.color[0] - from.color[0]) * t,
      from.color[1] + (to.color[1] - from.color[1]) * t,
      from.color[2] + (to.color[2] - from.color[2]) * t,
    ],
    environments: environments.pair,
    rotation: from.rotation + (to.rotation - from.rotation) * t,
    textures: skies.pair,
  };
}

/** The clouds walk with the skies, whose blend they share in the shader. */
function fadeClouds(from: IRendererSky, to: IRendererSky, progress: IWeatherWalk): Nullable<IRendererClouds> {
  const { t } = progress;

  if (!from.clouds || !to.clouds) {
    return to.clouds;
  }

  const textures: IWeatherSlots<Nullable<string>> = walk(
    { blend: from.blend, pair: from.clouds.textures },
    { blend: to.blend, pair: to.clouds.textures },
    progress
  );
  const [a, b] = [from.clouds.color, to.clouds.color];

  return {
    color: [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t],
    rotation: from.clouds.rotation + (to.clouds.rotation - from.clouds.rotation) * t,
    textures: textures.pair,
  };
}

/** How far a fade is, and whether its slots walk between pairs or only have their blend moved. */
interface IWeatherWalk {
  t: number;
  isWalked: boolean;
}

/**
 * Two slots walked from one pair to another: the first third leaves the old pair on its heavier half, the second
 * blends that half to the new pair's heavier one, the last brings the new pair to its own blend. Pairs that stay the
 * same only have their blend moved.
 */
function walk<T>(from: IWeatherSlots<T>, to: IWeatherSlots<T>, progress: IWeatherWalk): IWeatherSlots<T> {
  const { t, isWalked } = progress;

  if (!isWalked) {
    return { blend: from.blend + (to.blend - from.blend) * t, pair: to.pair };
  }

  const phase: number = t * 3;
  const fromHeavier: number = from.blend >= 0.5 ? 1 : 0;
  const toHeavier: number = to.blend >= 0.5 ? 1 : 0;

  if (phase < 1) {
    return { blend: from.blend + (fromHeavier - from.blend) * phase, pair: from.pair };
  }

  if (phase < 2) {
    return { blend: phase - 1, pair: [from.pair[fromHeavier], to.pair[toHeavier]] };
  }

  return { blend: toHeavier + (to.blend - toHeavier) * (phase - 2), pair: to.pair };
}

function isSamePair<T>(a: readonly [T, T], b: readonly [T, T]): boolean {
  return a[0] === b[0] && a[1] === b[1];
}

function normalise([x, y, z]: TRendererVector): TRendererVector {
  const length: number = Math.hypot(x, y, z) || 1;

  return [x / length, y / length, z / length];
}
