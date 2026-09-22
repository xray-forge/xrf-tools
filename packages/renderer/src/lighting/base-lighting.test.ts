import { describe, expect, it } from "@jest/globals";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { toBaseLightingConstants, toFogParams, toSunSpecular } from "#/lighting/base-lighting";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";

describe("toSunSpecular", () => {
  it("weights a dim light by the two-thirds power of its mean, times the gloss factor", () => {
    // Mean of noon's sun colour is 0.813072; 4 * 0.813072^(2/3).
    expect(toSunSpecular([0.905882, 0.839216, 0.694118])).toBeCloseTo(3.484544, 5);
  });

  it("weights a light brighter than one linearly", () => {
    expect(toSunSpecular([2, 2, 2])).toBe(8);
  });
});

describe("toFogParams", () => {
  it("ramps from 85% of the clear fraction to 99% of the distance", () => {
    // Noon: distance 350, density 0.9, so near = 29.75 and far = 346.5.
    const [offset, scale] = toFogParams(350, 0.9);

    expect(offset).toBeCloseTo(-0.093923, 6);
    expect(scale).toBeCloseTo(0.003157, 6);
  });
});

describe("toBaseLightingConstants", () => {
  it("doubles ambient and keeps it off zero", () => {
    const lighting: IRendererLighting = { ...DEFAULT_RENDERER_LIGHTING, ambientColor: [0.02, 0, 0.0001] };

    expect(toBaseLightingConstants(lighting).ambient).toEqual([0.04, 0.001, 0.001]);
  });

  it("doubles the hemisphere twice, once in the environment and once in combine", () => {
    expect(toBaseLightingConstants(DEFAULT_RENDERER_LIGHTING).environment[0]).toBeCloseTo(
      (0.470588 * 2 + 0.00001) * 2,
      6
    );
  });

  it("normalises the sun direction and binds no fog when there is none", () => {
    const constants = toBaseLightingConstants({ ...DEFAULT_RENDERER_LIGHTING, sunDirection: [0, -2, 0] });

    expect(constants.sunDirection).toEqual([0, -1, 0]);
    expect([constants.fogOffset, constants.fogScale]).toEqual([0, 0]);
  });
});
