import { describe, expect, it } from "@jest/globals";

import { CloudUniforms } from "#/uniforms/cloud-uniforms";

describe("CloudUniforms", () => {
  it("takes the colour as a colour byte holds it, and the turn in radians", () => {
    const clouds: CloudUniforms = new CloudUniforms();

    clouds.take({ color: [1.4, 0.5, -0.2, 0.8], rotation: 90, textures: ["a", "b"] });

    expect(clouds.color.value.toArray()).toEqual([1, 0.5, 0, 0.8]);
    expect(clouds.rotation.value).toBeCloseTo(Math.PI / 2, 10);
    expect(clouds.drawn.value).toBe(1);
  });

  // `RenderClouds` is not called at all under `EPS_L` of cover, whatever the textures are.
  it("draws nothing without cover, or without clouds", () => {
    const clouds: CloudUniforms = new CloudUniforms();

    clouds.take({ color: [1, 1, 1, 0.0005], rotation: 0, textures: ["a", "b"] });

    expect(clouds.drawn.value).toBe(0);

    clouds.take(null);

    expect(clouds.drawn.value).toBe(0);
  });
});
