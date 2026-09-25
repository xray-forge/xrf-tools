import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera } from "three/webgpu";

import { DEFAULT_RENDERER_GRASS_SETTINGS } from "#/contract/renderer-features";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

describe("GrassUniforms", () => {
  // `r__detail_radius` 49 is `dm_size` 24 and `dm_fade` 47.5; density 0.6 lays four steps, five candidates, each way.
  it("reaches as far and plants as densely as the engine at its defaults", () => {
    const uniforms: GrassUniforms = new GrassUniforms();

    uniforms.configure(DEFAULT_RENDERER_GRASS_SETTINGS);

    expect(uniforms.reach.value).toBe(24);
    expect(uniforms.fade.value).toBe(47.5);
    expect(uniforms.steps.value).toBe(4);
    expect(uniforms.jitter.value).toBeCloseTo(0.6 / 1.7, 8);
    expect(uniforms.slotCount).toBe(49 * 49);
    expect(uniforms.candidateCount).toBe(25);
  });

  it("stands the camera over its slot in the engine's space, rounded to the nearest", () => {
    const uniforms: GrassUniforms = new GrassUniforms();
    const camera: PerspectiveCamera = new PerspectiveCamera();

    camera.position.set(5.2, 3, 7.1);
    camera.updateMatrixWorld();
    uniforms.follow(camera, 10, 20, 3, 4);

    // Renderer `z` 7.1 is the engine's -7.1: `iFloor(-7.1 / 2 + 0.5)` is -4.
    expect(uniforms.eye.value.toArray()).toEqual([5.2, 3, -7.1]);
    expect(uniforms.center.value.x).toBe(3);
    expect(uniforms.center.value.y).toBe(-4);
    expect(uniforms.grid.value.toArray()).toEqual([10, 20, 3, 4]);
  });
});
