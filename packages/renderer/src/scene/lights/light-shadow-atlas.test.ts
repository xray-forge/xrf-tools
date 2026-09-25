import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { ILightShadowTile, LightShadowAtlas } from "#/scene/lights/light-shadow-atlas";

describe("LightShadowAtlas", () => {
  it("cuts a square into quarters for a smaller face, taking the least free square that holds it", () => {
    const atlas: LightShadowAtlas = new LightShadowAtlas(1024, 32);
    const large = atlas.allocate(512) as ILightShadowTile;
    const small = atlas.allocate(128) as ILightShadowTile;
    const next = atlas.allocate(128) as ILightShadowTile;

    expect(large).toEqual({ size: 512, x: 0, y: 0 });
    // Out of a quarter of the atlas, then out of what that left, before any other quarter is cut.
    expect(small.size).toBe(128);
    expect(next.size).toBe(128);
    expect(Math.floor(small.x / 512) + Math.floor(small.y / 512) * 2).toBe(
      Math.floor(next.x / 512) + Math.floor(next.y / 512) * 2
    );
    expect(atlas.used).toBe(512 * 512 + 2 * 128 * 128);
  });

  it("refuses a face once no free square is as large", () => {
    const atlas: LightShadowAtlas = new LightShadowAtlas(256, 32);
    const tiles: Array<Nullable<ILightShadowTile>> = [128, 128, 128, 128].map((size) => atlas.allocate(size));

    expect(tiles.every(Boolean)).toBe(true);
    expect(atlas.allocate(32)).toBeNull();
  });

  it("joins four free quarters back into their square", () => {
    const atlas: LightShadowAtlas = new LightShadowAtlas(256, 32);
    const tiles: Array<ILightShadowTile> = [32, 32, 64, 128].map((size) => atlas.allocate(size) as ILightShadowTile);

    tiles.forEach((tile: ILightShadowTile) => atlas.release(tile));

    expect(atlas.used).toBe(0);
    expect(atlas.allocate(256)).toEqual({ size: 256, x: 0, y: 0 });
  });
});
