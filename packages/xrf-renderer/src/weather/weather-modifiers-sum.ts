import { TRendererVector } from "#/contract/renderer-vector";
import { IRendererWeatherModifier } from "#/contract/weather/renderer-weather-modifier";

/** `EEnvModUsedParams`. */
export const WEATHER_MODIFIER_FLAGS = {
  AMBIENT_COLOR: 1 << 3,
  FAR_PLANE: 1 << 0,
  FOG_COLOR: 1 << 1,
  FOG_DENSITY: 1 << 2,
  HEMI_COLOR: 1 << 5,
  SKY_COLOR: 1 << 4,
} as const;

/**
 * Every modifier reaching a point, summed as `CEnvironment::lerp` sums them into one `CEnvModifier`.
 */
export interface IWeatherModifiersSum {
  farPlane: number;
  fogColor: TRendererVector;
  fogDensity: number;
  ambient: TRendererVector;
  skyColor: TRendererVector;
  hemiColor: TRendererVector;
  /** The values some modifier reaching the point adds to. */
  flags: number;
  /** Every reaching modifier's weight together, `mpower`. */
  power: number;
  /** How many reach it. */
  count: number;
}

/**
 * @param modifiers - The level's volumes.
 * @param view - The point, in engine space.
 * @returns What they add there, each weighted by its power falling off to nothing at its radius.
 */
export function toWeatherModifiersSum(
  modifiers: ReadonlyArray<IRendererWeatherModifier>,
  view: TRendererVector
): IWeatherModifiersSum {
  const fogColor: [number, number, number] = [0, 0, 0];
  const ambient: [number, number, number] = [0, 0, 0];
  const skyColor: [number, number, number] = [0, 0, 0];
  const hemiColor: [number, number, number] = [0, 0, 0];

  let farPlane: number = 0;
  let fogDensity: number = 0;
  let flags: number = 0;
  let power: number = 0;
  let count: number = 0;

  for (const modifier of modifiers) {
    const distance: number = Math.hypot(
      view[0] - modifier.position[0],
      view[1] - modifier.position[1],
      view[2] - modifier.position[2]
    );

    // `sum` returns before it sets any flag for a point outside the radius.
    if (distance * distance >= modifier.radius * modifier.radius) {
      continue;
    }

    const weight: number = modifier.power * (1 - distance / modifier.radius);

    function add(target: [number, number, number], value: TRendererVector, flag: number): void {
      if (modifier.flags & flag) {
        target[0] += value[0] * weight;
        target[1] += value[1] * weight;
        target[2] += value[2] * weight;
      }
    }

    if (modifier.flags & WEATHER_MODIFIER_FLAGS.FAR_PLANE) {
      farPlane += modifier.farPlane * weight;
    }

    if (modifier.flags & WEATHER_MODIFIER_FLAGS.FOG_DENSITY) {
      fogDensity += modifier.fogDensity * weight;
    }

    add(fogColor, modifier.fogColor, WEATHER_MODIFIER_FLAGS.FOG_COLOR);
    add(ambient, modifier.ambient, WEATHER_MODIFIER_FLAGS.AMBIENT_COLOR);
    add(skyColor, modifier.skyColor, WEATHER_MODIFIER_FLAGS.SKY_COLOR);
    add(hemiColor, modifier.hemiColor, WEATHER_MODIFIER_FLAGS.HEMI_COLOR);

    flags |= modifier.flags;
    power += weight;
    count += 1;
  }

  return { ambient, count, farPlane, flags, fogColor, fogDensity, hemiColor, power, skyColor };
}
