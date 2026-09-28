import { describe, expect, it } from "@jest/globals";

import { DEFAULT_RENDERER_TREE_WIND } from "#/lighting/default-lighting";
import { TreeWindUniforms } from "#/uniforms/tree-wind-uniforms";

describe("TreeWindUniforms", () => {
  // `FTreeVisual_setup::calculate`: a wind turning once every `rotation` seconds at the amplitude's length, and the
  // wave's direction and phase over a turn, the engine's `z` negated into renderer space.
  it("builds the engine's wind and wave for the time", () => {
    const uniforms: TreeWindUniforms = new TreeWindUniforms();

    uniforms.take(DEFAULT_RENDERER_TREE_WIND);
    uniforms.update(2.5);

    const turn: number = Math.PI * 2;
    const rotation: number = (turn * 2.5) / DEFAULT_RENDERER_TREE_WIND.rotation;

    expect(uniforms.wind.value.x).toBeCloseTo(Math.sin(rotation) * 0.005, 8);
    expect(uniforms.wind.value.y).toBe(0);
    expect(uniforms.wind.value.z).toBeCloseTo(-Math.cos(rotation) * 0.005, 8);
    expect(uniforms.wind.value.length()).toBeCloseTo(0.005, 8);
    expect(uniforms.wave.value.toArray()).toEqual([0.1 / turn, 0.01 / turn, -0.11 / turn, 2.5 / turn]);
    expect(uniforms.isSwaying).toBe(true);
  });

  it("keeps the frame before's wind and wave, where a tree's motion is measured from", () => {
    const uniforms: TreeWindUniforms = new TreeWindUniforms();

    uniforms.take(DEFAULT_RENDERER_TREE_WIND);
    uniforms.update(1);

    const wind = uniforms.wind.value.clone();
    const wave = uniforms.wave.value.clone();

    uniforms.update(1.5);

    expect(uniforms.previousWind.value.equals(wind)).toBe(true);
    expect(uniforms.previousWave.value.equals(wave)).toBe(true);
    expect(uniforms.wind.value.equals(wind)).toBe(false);
  });

  it("stands the trees still without a wind", () => {
    const uniforms: TreeWindUniforms = new TreeWindUniforms();

    uniforms.take(DEFAULT_RENDERER_TREE_WIND);
    uniforms.update(1);
    uniforms.take(null);
    uniforms.update(2);

    expect(uniforms.wind.value.toArray()).toEqual([0, 0, 0]);
    expect(uniforms.isSwaying).toBe(false);
  });
});
