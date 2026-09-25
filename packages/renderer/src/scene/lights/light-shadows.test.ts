import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Box3, Vector3 } from "three/webgpu";

import {
  ILightShadowEntry,
  ILightShadowRequest,
  LightShadows,
  toLightShadowSize,
  toLightShadowTileSize,
} from "#/scene/lights/light-shadows";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";

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

/** A box a metre across around a point. */
function createBox(x: number, y: number, z: number): Box3 {
  return new Box3(new Vector3(x - 0.5, y - 0.5, z - 0.5), new Vector3(x + 0.5, y + 0.5, z + 0.5));
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

/** Asks for each light, decides, and answers each one's faces. */
function plan(
  shadows: LightShadows,
  requests: ReadonlyArray<ILightShadowRequest>,
  budget: number = 8
): Array<Nullable<ILightShadowEntry>> {
  requests.forEach((request: ILightShadowRequest, index: number) => shadows.request(index, request));
  shadows.finish(budget);

  return requests.map((_, index: number) => shadows.getEntry(index));
}

describe("LightShadows", () => {
  it("draws the never drawn nearest first within the budget, then keeps a face until a change reaches it", () => {
    const shadows: LightShadows = new LightShadows();
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const lights: Array<ILightShadowRequest> = [
      createRequest({ distance: 30, position: new Vector3(100, 3, 0) }),
      createRequest({ distance: 1 }),
    ];

    shadows.begin(changes, false);

    const [far, near] = plan(shadows, lights, 1) as Array<ILightShadowEntry>;

    expect(shadows.queue).toEqual([near.faces[0]]);
    expect(shadows.isReady(near)).toBe(true);
    expect(shadows.isReady(far)).toBe(false);

    shadows.markDrawn();
    shadows.begin(changes, false);
    plan(shadows, lights);
    shadows.markDrawn();

    expect(shadows.queue).toEqual([far.faces[0]]);

    // A caster comes under the near spot only: the far one's face stays as it is.
    changes.put(7, createBox(0, 0, 0), true, false);
    shadows.begin(changes, false);
    plan(shadows, lights);

    expect(shadows.queue).toEqual([near.faces[0]]);

    // A caster whose place is not known changes every face.
    shadows.markDrawn();
    changes.put(8, null, true, false);
    shadows.begin(changes, false);
    plan(shadows, lights);

    expect(shadows.queue).toEqual([near.faces[0], far.faces[0]]);
  });

  it("draws a face a swaying caster stands in again every frame the wind blows", () => {
    const shadows: LightShadows = new LightShadows();
    const changes: StaticShadowChanges = new StaticShadowChanges();

    changes.put(1, createBox(0, 0, 0), true, true);
    shadows.begin(changes, true);

    const [entry] = plan(shadows, [createRequest()]) as Array<ILightShadowEntry>;

    shadows.markDrawn();
    shadows.begin(changes, true);
    plan(shadows, [createRequest()]);

    expect(shadows.queue).toEqual([entry.faces[0]]);

    shadows.markDrawn();
    shadows.begin(changes, false);
    plan(shadows, [createRequest()]);

    expect(shadows.queue).toEqual([]);
  });

  it("gives a point light six faces looking along the world's axes", () => {
    const shadows: LightShadows = new LightShadows();

    shadows.begin(new StaticShadowChanges(), false);

    const [entry] = plan(shadows, [createRequest({ isSpot: false })]) as Array<ILightShadowEntry>;
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

  it("never gives one light in view another's squares, and asks every face for smaller once they want too much", () => {
    const shadows: LightShadows = new LightShadows();
    const changes: StaticShadowChanges = new StaticShadowChanges();
    // A 1024 face each: sixteen fill the atlas, and the seventeenth, the farthest, finds no room.
    const lights: Array<ILightShadowRequest> = Array.from({ length: 17 }, (_, index: number) =>
      createRequest({ distance: index, range: 200 })
    );

    shadows.begin(changes, false);

    const first: Array<Nullable<ILightShadowEntry>> = plan(shadows, lights);

    expect(first.slice(0, 16).every((it) => it?.size === 1024)).toBe(true);
    expect(first[16]).toBeNull();
    // A light in view found no room: every face is asked for less from the next frame.
    expect(shadows.sizeScale).toBeCloseTo(0.7, 6);

    let entries: Array<Nullable<ILightShadowEntry>> = first;

    for (let frame: number = 0; frame < 4 && entries.some((it) => it === null); frame += 1) {
      shadows.markDrawn();
      shadows.begin(changes, false);
      entries = plan(shadows, lights);
    }

    // Small enough for every light to have room, each drawn again in its new square.
    expect(entries.every((it) => it?.size === 512)).toBe(true);
    expect(shadows.sizeScale).toBeLessThan(0.7);
  });

  it("makes room from a light out of view", () => {
    const shadows: LightShadows = new LightShadows();
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const large: ILightShadowRequest = createRequest({ range: 200 });

    shadows.begin(changes, false);
    plan(
      shadows,
      Array.from({ length: 16 }, () => large)
    );
    shadows.begin(changes, false);
    // Light 16 alone in view: the lights out of view give it their room.
    shadows.request(16, large);
    shadows.finish(8);

    expect(shadows.getEntry(16)?.size).toBe(1024);
    expect(shadows.getEntry(0)).toBeNull();
  });
});
