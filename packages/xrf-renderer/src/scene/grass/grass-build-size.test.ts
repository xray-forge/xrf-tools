import { describe, expect, it } from "@jest/globals";

import { DEFAULT_RENDERER_GRASS_SETTINGS } from "#/contract/renderer-grass-settings";
import { IGrassBuildSize, isGrassBuildOutgrown, toGrassBuildSize } from "#/scene/grass/grass-build-size";
import { GrassUniforms } from "#/uniforms/grass-uniforms";

describe("toGrassBuildSize", () => {
  it("holds a cell a slot the planting reaches, every candidate each, and a band a ring and the camera's", () => {
    const uniforms: GrassUniforms = new GrassUniforms();

    uniforms.configure(DEFAULT_RENDERER_GRASS_SETTINGS);

    // `dm_size` 24 at the default radius: a line of 49 slots, five candidates each way.
    expect(toGrassBuildSize(uniforms, 1 << 27)).toEqual({ bands: 25, capacity: 65_536, cells: 49 * 49, perCell: 25 });
  });
});

describe("isGrassBuildOutgrown", () => {
  const held: IGrassBuildSize = { bands: 25, capacity: 65_536, cells: 2401, perCell: 25 };

  it("holds any want within it, a smaller ring included", () => {
    expect(isGrassBuildOutgrown(held, held)).toBe(false);
    expect(isGrassBuildOutgrown(held, { bands: 13, capacity: 16_384, cells: 625, perCell: 25 })).toBe(false);
  });

  it("builds again for a want past it on any count", () => {
    expect(isGrassBuildOutgrown(held, { ...held, cells: 2402 })).toBe(true);
    expect(isGrassBuildOutgrown(held, { ...held, perCell: 36 })).toBe(true);
    expect(isGrassBuildOutgrown(held, { ...held, capacity: 131_072 })).toBe(true);
    expect(isGrassBuildOutgrown(held, { ...held, bands: 26 })).toBe(true);
  });
});
