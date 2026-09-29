import { describe, expect, it } from "@jest/globals";

import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { SkyUniforms } from "#/uniforms/sky-uniforms";

describe("SkyUniforms", () => {
  it("reads the lighting's sky, its rotation in radians, its curve, and whether the frame draws it", () => {
    const sky: SkyUniforms = new SkyUniforms();

    sky.take({ ...DEFAULT_RENDERER_LIGHTING.sky, blend: 0.25, color: [0.5, 0.6, 0.7], rotation: 90 });
    sky.setCurved(true);
    sky.setDrawn(true);
    sky.setHazed(true);

    expect(sky.blend.value).toBe(0.25);
    expect(sky.color.value.toArray()).toEqual([0.5, 0.6, 0.7]);
    expect(sky.rotation.value).toBeCloseTo(Math.PI / 2);
    expect(sky.drawn.value).toBe(1);
    expect(sky.hazed.value).toBe(1);
    expect(sky.curved.value).toBe(1);
  });

  // The pipelines that sample the sky bind cubes, whatever the lighting names later.
  it("samples a cube before anything is put", () => {
    expect(new SkyUniforms().cubes.every((cube) => (cube.value as { isCubeTexture?: boolean }).isCubeTexture)).toBe(
      true
    );
  });
});
