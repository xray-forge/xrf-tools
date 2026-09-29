import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { ERendererEngine } from "#/contract/renderer-engine";
import { IRendererSunPosition } from "#/contract/weather/renderer-sun-position";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IRendererWeatherModifier } from "#/contract/weather/renderer-weather-modifier";
import { IWeatherCycleMix } from "#/weather/weather-cycle-mix";
import { readWeatherMixGolden } from "#/weather/weather-mix-fixtures";
import { mixWeather, selectWeatherKeyframes, weighWeatherTime } from "#/weather/weather-mixer";
import { EWeatherSun, TWeatherSun } from "#/weather/weather-sun";

/** One cycle `xrf-environment` mixed through a day. */
interface IGoldenCase {
  name: string;
  engine: ERendererEngine;
  sun: EWeatherSun;
  keyframes: Array<IRendererWeatherKeyframe>;
  sunTable: Nullable<Array<IRendererSunPosition>>;
  modifiers: Array<IRendererWeatherModifier>;
  mixes: Array<IWeatherCycleMix>;
}

const GOLDEN: ReadonlyArray<IGoldenCase> = readWeatherMixGolden<ReadonlyArray<IGoldenCase>>();

/** The Rust mixer works in `f32`, as the engine does. */
function expectClose(actual: unknown, expected: unknown, path: string): void {
  if (typeof expected === "number") {
    expect([path, Math.abs((actual as number) - expected) <= 1e-4 * Math.max(1, Math.abs(expected))]).toEqual([
      path,
      true,
    ]);
  } else if (expected === null || typeof expected !== "object") {
    expect([path, actual]).toEqual([path, expected]);
  } else if (Array.isArray(expected)) {
    expected.forEach((value: unknown, index: number) =>
      expectClose((actual as Array<unknown>)[index], value, `${path}[${index}]`)
    );
  } else {
    Object.entries(expected as Record<string, unknown>).forEach(([key, value]: [string, unknown]) =>
      expectClose((actual as Record<string, unknown>)[key], value, `${path}.${key}`)
    );
  }
}

function toSun(golden: IGoldenCase): TWeatherSun {
  return golden.sun === EWeatherSun.TABLE
    ? { kind: EWeatherSun.TABLE, positions: golden.sunTable ?? [] }
    : { kind: golden.sun };
}

describe("mixWeather", () => {
  it.each(GOLDEN.map((golden: IGoldenCase) => [golden.name, golden] as const))(
    "mixes %s as xrf-environment does",
    (_: string, golden: IGoldenCase) => {
      expect(golden.mixes.length).toBeGreaterThan(0);

      for (const expected of golden.mixes) {
        const actual: Nullable<IWeatherCycleMix> = mixWeather(
          { engine: golden.engine, keyframes: golden.keyframes, modifiers: golden.modifiers, sun: toSun(golden) },
          { time: expected.time, view: expected.view }
        );

        expect(actual?.keyframes).toEqual(expected.keyframes);
        expectClose(actual, expected, `${golden.name} at ${expected.time} from ${expected.view.join(", ")}`);
      }
    }
  );

  it("mixes nothing for a cycle without keyframes", () => {
    expect(
      mixWeather(
        { engine: ERendererEngine.VANILLA, keyframes: [], modifiers: [], sun: { kind: EWeatherSun.AUTHORED } },
        { time: 0, view: [0, 0, 0] }
      )
    ).toBeNull();
  });
});

describe("selectWeatherKeyframes", () => {
  it("wraps around midnight either side of the day's keyframes", () => {
    const keyframes: Array<IRendererWeatherKeyframe> = GOLDEN[0].keyframes;

    expect(selectWeatherKeyframes(keyframes, 0)).toEqual([keyframes.length - 1, 0]);
    expect(selectWeatherKeyframes(keyframes, 86_399)).toEqual([keyframes.length - 1, 0]);
  });
});

describe("weighWeatherTime", () => {
  it("weighs across midnight", () => {
    expect(weighWeatherTime(0, [82_800, 3_600])).toBeCloseTo(0.5, 10);
    expect(weighWeatherTime(40_000, [82_800, 3_600])).toBe(0);
  });

  it("weighs a span of nothing at nothing", () => {
    expect(weighWeatherTime(100, [100, 100])).toBe(0);
  });
});
