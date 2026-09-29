import { describe, expect, it } from "@jest/globals";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererSky } from "#/contract/renderer-sky";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { toFadedLighting } from "#/weather/weather-fade";

function toLighting(sky: Partial<IRendererSky>, overrides: Partial<IRendererLighting> = {}): IRendererLighting {
  return {
    ...DEFAULT_RENDERER_LIGHTING,
    sky: { ...DEFAULT_RENDERER_LIGHTING.sky, clouds: null, ...sky },
    ...overrides,
  };
}

/** What a sky slot shows: each cube weighed by the blend, those the pair names twice added. */
function toShown(sky: IRendererSky): Record<string, number> {
  const shown: Record<string, number> = {};

  shown[sky.textures[0]] = (shown[sky.textures[0]] ?? 0) + (1 - sky.blend);
  shown[sky.textures[1]] = (shown[sky.textures[1]] ?? 0) + sky.blend;

  return Object.fromEntries(Object.entries(shown).filter(([, weight]) => weight > 1e-9));
}

describe("toFadedLighting", () => {
  it("blends every value from what was shown to what is shown now, and is what is shown now once done", () => {
    const from: IRendererLighting = toLighting({}, { ambientColor: [0, 0, 0], waterIntensity: 0 });
    const to: IRendererLighting = toLighting({}, { ambientColor: [1, 0.5, 0], waterIntensity: 1 });
    const half: IRendererLighting = toFadedLighting({ from, progress: 0.5, to });

    expect(half.ambientColor).toEqual([0.5, 0.25, 0]);
    expect(half.waterIntensity).toBe(0.5);
    expect(toFadedLighting({ from, progress: 1, to })).toBe(to);
  });

  // The sky has two slots and one blend: every step shows only cubes the step before showed, or a blend of them.
  it("walks the skies from one pair to the other without a cube appearing out of nothing", () => {
    const from: IRendererLighting = toLighting({ blend: 0.3, textures: ["a", "b"] });
    const to: IRendererLighting = toLighting({ blend: 0.8, textures: ["c", "d"] });
    const steps: Array<Record<string, number>> = Array.from({ length: 31 }, (_: unknown, at: number) =>
      toShown(toFadedLighting({ from, progress: at / 30, to }).sky)
    );

    expect(steps[0].a).toBeCloseTo(0.7, 10);
    expect(steps[0].b).toBeCloseTo(0.3, 10);
    // A third in, the old pair is down to its heavier half; two thirds in, only the new pair's heavier half is left.
    expect(steps[10]).toEqual({ a: 1 });
    expect(steps[20]).toEqual({ d: 1 });

    for (let at: number = 1; at < steps.length; at += 1) {
      const before: Record<string, number> = steps[at - 1];
      const now: Record<string, number> = steps[at];
      // A cube drawn now and not a step ago comes in from nothing.
      const appeared: Array<number> = Object.keys(now)
        .filter((key: string) => before[key] === undefined)
        .map((key: string) => now[key]);

      expect(appeared.every((weight: number) => weight < 0.1)).toBe(true);
    }
  });

  it("only moves the blend where the pair stays the same", () => {
    const from: IRendererLighting = toLighting({ blend: 0.2, textures: ["a", "b"] });
    const to: IRendererLighting = toLighting({ blend: 0.6, textures: ["a", "b"] });

    expect(toFadedLighting({ from, progress: 0.5, to }).sky).toMatchObject({ blend: 0.4, textures: ["a", "b"] });
  });

  it("fades rain in and out from nothing, and stops it under the engine's threshold", () => {
    const dry: IRendererLighting = toLighting({}, { rain: null });
    const wet: IRendererLighting = toLighting(
      {},
      { rain: { color: [1, 1, 1], density: 0.8, windDirection: 0, windVelocity: 0 } }
    );

    expect(toFadedLighting({ from: dry, progress: 0.5, to: wet }).rain?.density).toBeCloseTo(0.4, 10);
    expect(toFadedLighting({ from: wet, progress: 0.5, to: dry }).rain?.density).toBeCloseTo(0.4, 10);
    expect(toFadedLighting({ from: wet, progress: 0.9995, to: dry }).rain).toBeNull();
  });
});
