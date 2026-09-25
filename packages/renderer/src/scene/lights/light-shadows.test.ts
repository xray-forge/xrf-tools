import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Vector3 } from "three/webgpu";

import {
  ILightShadowEntry,
  ILightShadowRequest,
  LightShadows,
  toLightShadowSize,
  toLightShadowTileSize,
} from "#/scene/lights/light-shadows";

function createRequest(part: Partial<ILightShadowRequest> = {}): ILightShadowRequest {
  return {
    cone: Math.PI / 2,
    direction: new Vector3(0, -1, 0),
    distance: 0,
    duel: 1,
    intensity: 1,
    isSpot: true,
    near: 0.1,
    position: new Vector3(0, 3, 0),
    range: 8,
    up: new Vector3(0, 0, 1),
    ...part,
  };
}

describe("toLightShadowSize", () => {
  it("sizes a map as `compute_xf_spot` does: the optimal 768 for an eight metre, quarter turn spot at hand", () => {
    expect(toLightShadowSize(createRequest())).toBe(768);
    // Twice as far off as it reaches: its area falls to a quarter, the map to half.
    expect(toLightShadowSize(createRequest({ distance: Math.sqrt(4 * 64 - 1) }))).toBe(384);
    expect(toLightShadowSize(createRequest({ distance: 1000 }))).toBe(32);
    expect(toLightShadowSize(createRequest({ range: 200 }))).toBe(1536);
  });
});

describe("toLightShadowTileSize", () => {
  it("takes the nearest power of two, and keeps a square while the size stays within half a square of it", () => {
    expect(toLightShadowTileSize(768, 0)).toBe(1024);
    expect(toLightShadowTileSize(700, 512)).toBe(512);
    expect(toLightShadowTileSize(800, 512)).toBe(1024);
    expect(toLightShadowTileSize(320, 512)).toBe(256);
    expect(toLightShadowTileSize(10, 0)).toBe(32);
    expect(toLightShadowTileSize(1536, 0)).toBe(1024);
  });
});

describe("LightShadows", () => {
  it("draws a spot's face once, the never drawn nearest first within the budget, and again once its casters change", () => {
    const shadows: LightShadows = new LightShadows();

    shadows.begin(1);

    const far: Nullable<ILightShadowEntry> = shadows.request(0, createRequest({ distance: 30 }));
    const near: Nullable<ILightShadowEntry> = shadows.request(1, createRequest({ distance: 1 }));

    shadows.finish(1);

    expect(shadows.queue).toEqual([near?.faces[0]]);
    expect(shadows.isReady(near as ILightShadowEntry)).toBe(true);
    expect(shadows.isReady(far as ILightShadowEntry)).toBe(false);

    shadows.markDrawn();
    shadows.begin(1);
    shadows.request(0, createRequest({ distance: 30 }));
    shadows.request(1, createRequest({ distance: 1 }));
    shadows.finish(8);

    expect(shadows.queue).toEqual([far?.faces[0]]);

    shadows.markDrawn();
    shadows.begin(2);
    shadows.request(0, createRequest({ distance: 30 }));
    shadows.request(1, createRequest({ distance: 1 }));
    shadows.finish(8);

    // Stale, both still shadow while they are drawn again.
    expect(shadows.queue).toEqual([near?.faces[0], far?.faces[0]]);
    expect(shadows.isReady(far as ILightShadowEntry)).toBe(true);
  });

  it("gives a point light six faces looking along the world's axes", () => {
    const shadows: LightShadows = new LightShadows();

    shadows.begin(0);

    const entry = shadows.request(0, createRequest({ isSpot: false })) as ILightShadowEntry;
    const looking = entry.faces.map((face) =>
      face.camera
        .getWorldDirection(new Vector3())
        .toArray()
        .map((it) => Math.round(it) + 0)
    );

    expect(looking).toEqual([
      [1, 0, 0],
      [-1, 0, 0],
      [0, 1, 0],
      [0, -1, 0],
      [0, 0, 1],
      [0, 0, -1],
    ]);
    expect(entry.near).toBe(0.1);
    expect(entry.far).toBeCloseTo(8, 6);
  });

  it("makes room from the lights out of view longest, and never from one in view this frame", () => {
    const shadows: LightShadows = new LightShadows();
    // A 1024 face each: sixteen fill the atlas.

    function large(): ILightShadowRequest {
      return createRequest({ range: 200 });
    }

    shadows.begin(0);

    for (let index: number = 0; index < 16; index += 1) {
      expect(shadows.request(index, large())).not.toBeNull();
    }

    // Every square is held by a light in view: the next gets a smaller one or none.
    expect(shadows.request(16, large())).toBeNull();

    shadows.begin(0);

    // Light 0 was out of view this frame, so the new light takes its square.
    expect(shadows.request(17, large())?.size).toBe(1024);
    expect(shadows.atlas.used).toBe(4096 * 4096);
  });
});
