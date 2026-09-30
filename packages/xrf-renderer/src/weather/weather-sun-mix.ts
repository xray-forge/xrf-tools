import { TRendererVector } from "#/contract/renderer-vector";
import { IDynamicSun, toDynamicSun } from "#/weather/dynamic-sun";
import { toSunTableDirection } from "#/weather/sun-table-direction";
import { IWeatherMixedSun } from "#/weather/weather-mixed-sun";
import { EWeatherSun } from "#/weather/weather-sun";
import { IWeatherSunMixInput } from "#/weather/weather-sun-mix-input";

/** Where an authored sun stands for a keyframe that stands none. */
const DOWN: TRendererVector = [0, -1, 0];

/**
 * The sun `CEnvDescriptorMixer::lerp` stands: the keyframes' own directions blended, the astronomical sun, or
 * Monolith's sun table, its colour blended either way.
 *
 * @param input - Where the sun is stood from, the pair, and when between them.
 * @returns The sun.
 */
export function mixWeatherSun(input: IWeatherSunMixInput): IWeatherMixedSun {
  const { sun, pair, time, weight } = input;
  const [a, b] = pair;

  function lerp(from: TRendererVector, to: TRendererVector): TRendererVector {
    return [0, 1, 2].map((axis: number) => (1 - weight) * from[axis] + weight * to[axis]) as unknown as TRendererVector;
  }

  const color: TRendererVector = lerp(a.sunColor, b.sunColor);

  switch (sun.kind) {
    case EWeatherSun.AUTHORED:
      return { color, direction: normalise(lerp(a.sunDirection ?? DOWN, b.sunDirection ?? DOWN)) };

    // The engine passes the mixed `exec_time`, which runs backwards across midnight; the time of day it stands for is
    // the same everywhere else.
    case EWeatherSun.DYNAMIC: {
      const dynamic: IDynamicSun = toDynamicSun(time, (1 - weight) * a.sunAzimuth + weight * b.sunAzimuth);

      return {
        color: [color[0] * dynamic.blend, color[1] * dynamic.blend, color[2] * dynamic.blend],
        direction: dynamic.direction,
      };
    }

    case EWeatherSun.TABLE:
      return { color, direction: toSunTableDirection(sun.positions, time) };
  }
}

function normalise([x, y, z]: TRendererVector): TRendererVector {
  const length: number = Math.hypot(x, y, z);

  return length > 0 ? [x / length, y / length, z / length] : DOWN;
}
