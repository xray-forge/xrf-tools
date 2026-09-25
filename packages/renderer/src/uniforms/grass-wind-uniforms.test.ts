import { describe, expect, it } from "@jest/globals";

import { DEFAULT_RENDERER_GRASS_WIND } from "#/lighting/default-lighting";
import { GrassWindUniforms } from "#/uniforms/grass-wind-uniforms";

describe("GrassWindUniforms", () => {
  // Halfway between the normal and fast swings, the first wind turning once in 17.5 seconds at 0.225.
  it("turns each wind at the mixed swing's rate and length, and runs the waves by its speed", () => {
    const uniforms: GrassWindUniforms = new GrassWindUniforms();
    const turn: number = Math.PI * 2;

    uniforms.take(DEFAULT_RENDERER_GRASS_WIND);
    uniforms.update(3);

    const angle: number = (turn * 3) / 17.5;

    expect(uniforms.wind1.value.length()).toBeCloseTo(0.225, 8);
    expect(uniforms.wind1.value.x).toBeCloseTo(Math.sin(angle) * 0.225, 8);
    expect(uniforms.wind1.value.z).toBeCloseTo(Math.cos(angle) * 0.225, 8);
    expect(uniforms.wind2.value.length()).toBeCloseTo(0.125, 8);
    [1 / 5, 1 / 7, 1 / 3, 3 * 1.25].forEach((value: number, index: number) =>
      expect(uniforms.wave1.value.getComponent(index)).toBeCloseTo(value / turn, 10)
    );
    expect(uniforms.wave2.value.x).toBeCloseTo(1 / 3 / turn, 10);
  });

  it("keeps the frame before's, and stands the grass still without a wind", () => {
    const uniforms: GrassWindUniforms = new GrassWindUniforms();

    uniforms.take(DEFAULT_RENDERER_GRASS_WIND);
    uniforms.update(1);

    const wind = uniforms.wind1.value.clone();

    uniforms.take(null);
    uniforms.update(2);

    expect(uniforms.previousWind1.value.equals(wind)).toBe(true);
    expect(uniforms.wind1.value.toArray()).toEqual([0, 0, 0]);
  });
});
