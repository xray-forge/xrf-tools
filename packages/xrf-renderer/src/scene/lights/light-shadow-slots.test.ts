import { describe, expect, it } from "@jest/globals";
import { Sphere } from "three/webgpu";

import { LightShadowAtlas } from "#/scene/lights/light-shadow-atlas";
import { ILightShadowFace } from "#/scene/lights/light-shadow-face";
import { LightShadowSlots } from "#/scene/lights/light-shadow-slots";
import { ILightShadowTile } from "#/scene/lights/light-shadow-tile";

/** Has a light show faces over squares, last asked for in a frame. */
function hold(slots: LightShadowSlots, index: number, tiles: ReadonlyArray<ILightShadowTile>, seen: number): void {
  slots.take(index).shown = {
    asked: tiles[0].size,
    far: 1,
    faces: tiles.map((tile: ILightShadowTile) => ({ tile }) as unknown as ILightShadowFace),
    near: 0.1,
    seen,
    size: tiles[0].size,
    sphere: new Sphere(),
  };
}

describe("LightShadowSlots", () => {
  it("evicts nothing for a size the evictions would not make room for, and takes a smaller one without them", () => {
    const atlas: LightShadowAtlas = new LightShadowAtlas(4096);
    const slots: LightShadowSlots = new LightShadowSlots(atlas);

    // Out of view since the first frame: a 1024 square.
    hold(slots, 0, [atlas.allocate(1024) as ILightShadowTile], 1);

    // In view: the rest in 512s, one given back from each of four 1024 squares. Free and evictable, the atlas has the
    // area for two 1024s, but only one whole.
    const small: Array<ILightShadowTile> = Array.from({ length: 60 }, () => atlas.allocate(512) as ILightShadowTile);
    const parents: Set<string> = new Set();
    const freed: Array<ILightShadowTile> = small.filter((tile: ILightShadowTile) => {
      const parent: string = `${Math.floor(tile.x / 1024)}:${Math.floor(tile.y / 1024)}`;

      return parents.size < 4 && !parents.has(parent) && Boolean(parents.add(parent));
    });

    freed.forEach((tile: ILightShadowTile) => atlas.release(tile));
    hold(
      slots,
      1,
      small.filter((tile: ILightShadowTile) => !freed.includes(tile)),
      2
    );

    const used: number = atlas.used;

    expect(slots.allocate(2, 1024, 2)).toBeNull();
    expect(slots.get(0)).toBeDefined();
    expect(atlas.used).toBe(used);

    expect(slots.allocate(2, 512, 2)).toHaveLength(2);
    expect(slots.get(0)).toBeDefined();
  });

  it("makes room from the light out of view longest once that is enough", () => {
    const atlas: LightShadowAtlas = new LightShadowAtlas(2048);
    const slots: LightShadowSlots = new LightShadowSlots(atlas);

    hold(slots, 0, [atlas.allocate(1024) as ILightShadowTile], 1);
    hold(slots, 1, [atlas.allocate(1024) as ILightShadowTile], 2);
    hold(slots, 2, [atlas.allocate(1024) as ILightShadowTile, atlas.allocate(1024) as ILightShadowTile], 3);

    expect(slots.allocate(1, 1024, 3)).toHaveLength(1);
    expect(slots.get(0)).toBeUndefined();
    expect(slots.get(1)).toBeDefined();
  });
});
