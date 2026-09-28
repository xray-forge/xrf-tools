import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";
import { Box3, Vector3, Vector4 } from "three/webgpu";

import { ERendererLightKind } from "#/contract/scene/renderer-light";
import { createLightBasis, ILightBasis, toLightBasis } from "#/scene/lights/light-geometry";
import { ILightShadowEntry } from "#/scene/lights/light-shadow-entry";
import { ILightShadowFace } from "#/scene/lights/light-shadow-face";
import {
  LIGHT_SHADOW_POINT_CONE,
  LIGHT_SHADOW_POINT_FACES,
  toLightShadowScale,
} from "#/scene/lights/light-shadow-faces";
import { LightShadowPlanner } from "#/scene/lights/light-shadow-planner";
import { ILightShadowRequest } from "#/scene/lights/light-shadow-request";
import { toLightShadowSize, toLightShadowTileSize } from "#/scene/lights/light-shadow-sizing";
import { EShadowCasterMotion } from "#/scene/static/shadow-caster-motion";
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

/** One frame of the planner: begun, every light asked for, decided; each one's faces answered, then drawn. */
function plan(
  planner: LightShadowPlanner,
  requests: ReadonlyArray<ILightShadowRequest>,
  { budget = 8, isWindy = false, isDrawn = true }: { budget?: number; isWindy?: boolean; isDrawn?: boolean } = {}
): Array<Nullable<ILightShadowEntry>> {
  planner.begin(isWindy);
  requests.forEach((request: ILightShadowRequest, index: number) => planner.request(index, request));
  planner.finish(budget);

  const entries: Array<Nullable<ILightShadowEntry>> = requests.map((_, index: number) => planner.getEntry(index));

  if (isDrawn) {
    planner.markDrawn();
  }

  return entries;
}

/** Where a world point lands on a face, as its projection puts it: from zero to one, `v` down. */
function toProjectedUv(face: ILightShadowFace, point: Vector3): [number, number] {
  const clip: Vector4 = new Vector4(point.x, point.y, point.z, 1).applyMatrix4(face.view).applyMatrix4(face.projection);

  return [(clip.x / clip.w) * 0.5 + 0.5, 0.5 - (clip.y / clip.w) * 0.5];
}

/** Where the lights pass looks a point up, a CPU mirror of its face pick and basis: the face and its uv. */
function toLookedUpUv(isSpot: boolean, basis: ILightBasis, toPoint: Vector3, cone: number): [number, number, number] {
  let [right, up, direction] = [basis.right, basis.up, basis.direction];
  let face: number = 0;
  let scale: number = toLightShadowScale(cone);

  if (!isSpot) {
    const [x, y, z] = [Math.abs(toPoint.x), Math.abs(toPoint.y), Math.abs(toPoint.z)];

    face = x >= y && x >= z ? (toPoint.x > 0 ? 0 : 1) : y >= z ? (toPoint.y > 0 ? 2 : 3) : toPoint.z > 0 ? 4 : 5;

    const basisFace = LIGHT_SHADOW_POINT_FACES[face];

    direction = new Vector3(...basisFace.direction);
    up = new Vector3(...basisFace.up);
    right = new Vector3().crossVectors(direction, up);
    scale = toLightShadowScale(LIGHT_SHADOW_POINT_CONE);
  }

  const along: number = toPoint.dot(direction);

  return [face, 0.5 + ((toPoint.dot(right) * scale) / along) * 0.5, 0.5 - ((toPoint.dot(up) * scale) / along) * 0.5];
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
  it("takes the nearest power of two, and keeps the square asked while the size stays within its band", () => {
    expect(toLightShadowTileSize(768, 0)).toBe(1024);
    expect(toLightShadowTileSize(700, 512)).toBe(512);
    expect(toLightShadowTileSize(800, 512)).toBe(1024);
    expect(toLightShadowTileSize(320, 512)).toBe(256);
    expect(toLightShadowTileSize(10, 0)).toBe(32);
    expect(toLightShadowTileSize(1536, 0)).toBe(1024);
  });

  it("takes the power of two at or below the size while the atlas is short of room, whatever it held", () => {
    expect(toLightShadowTileSize(1000, 0, true)).toBe(512);
    expect(toLightShadowTileSize(1000, 1024, true)).toBe(512);
    expect(toLightShadowTileSize(1100, 0, true)).toBe(1024);
    expect(toLightShadowTileSize(20, 0, true)).toBe(32);
  });
});

describe("LightShadowPlanner", () => {
  it("draws the never drawn nearest first within the budget, then keeps a face until a change reaches it", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const planner: LightShadowPlanner = new LightShadowPlanner(changes);
    const lights: Array<ILightShadowRequest> = [
      createRequest({ distance: 30, position: new Vector3(100, 3, 0) }),
      createRequest({ distance: 1 }),
    ];

    const [far, near] = plan(planner, lights, { budget: 1 });

    // The far light's face is not drawn yet: it waits, rather than lighting unshadowed.
    expect(far).toBeNull();
    expect(near).not.toBeNull();

    const [drawnFar] = plan(planner, lights);

    expect(drawnFar).not.toBeNull();

    // A caster comes under the near spot only: the far one's face stays as it is.
    changes.put(7, createBox(0, 0, 0), true);
    plan(planner, lights, { isDrawn: false });

    expect(planner.queue).toEqual([(near as ILightShadowEntry).faces[0]]);

    // A caster whose place is not known changes every face.
    planner.markDrawn();
    changes.put(8, null, true);
    plan(planner, lights, { isDrawn: false });

    expect(planner.queue).toEqual([(near as ILightShadowEntry).faces[0], (drawnFar as ILightShadowEntry).faces[0]]);
  });

  it("draws a face a swaying caster stands in again every frame the wind blows, a moving one's every frame", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const planner: LightShadowPlanner = new LightShadowPlanner(changes);

    changes.put(1, createBox(0, 0, 0), true, EShadowCasterMotion.SWAYING);
    plan(planner, [createRequest()], { isWindy: true });
    plan(planner, [createRequest()], { isDrawn: false, isWindy: true });

    expect(planner.queue).toHaveLength(1);

    planner.markDrawn();
    plan(planner, [createRequest()], { isDrawn: false });

    expect(planner.queue).toHaveLength(0);

    // A skinned part comes under it: drawn again whatever the wind does.
    changes.put({}, createBox(0, 1, 0), true, EShadowCasterMotion.MOVING);
    plan(planner, [createRequest()]);
    plan(planner, [createRequest()], { isDrawn: false });

    expect(planner.queue).toHaveLength(1);
  });

  // A level arriving logs thousands of swaying casters in one frame: each face's motion is found once, not once each.
  it("finds a face's motion once a frame however many swaying casters came under it", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const planner: LightShadowPlanner = new LightShadowPlanner(changes);

    plan(planner, [createRequest()], { isWindy: true });

    const getMotion = jest.spyOn(changes, "getMotion");

    for (let caster: number = 0; caster < 50; caster += 1) {
      changes.put(caster, createBox(0, 0, 0), true, EShadowCasterMotion.SWAYING);
    }

    plan(planner, [createRequest()], { isDrawn: false, isWindy: true });

    // A spot's one face, found once.
    expect(getMotion).toHaveBeenCalledTimes(1);
    expect(planner.queue).toHaveLength(1);
  });

  it("draws a stale face before every face over what sways, which take turns by how long ago they were drawn", () => {
    const changes: StaticShadowChanges = new StaticShadowChanges();
    const planner: LightShadowPlanner = new LightShadowPlanner(changes);
    const everywhere: Box3 = new Box3(new Vector3(-1000, -1000, -1000), new Vector3(1000, 1000, 1000));
    const lights: Array<ILightShadowRequest> = Array.from({ length: 12 }, (_, index: number) =>
      createRequest({ distance: index, position: new Vector3(index * 50, 3, 0) })
    );

    changes.put(1, everywhere, true, EShadowCasterMotion.SWAYING);

    for (let frame: number = 0; frame < 3; frame += 1) {
      plan(planner, lights, { isWindy: true });
    }

    // A change under the farthest light alone, while all twelve sway.
    changes.put(2, createBox(550, 1, 0), true);

    const [farthest] = plan(planner, lights, { isDrawn: false, isWindy: true }).slice(-1) as Array<ILightShadowEntry>;

    expect(planner.queue[0]).toBe(farthest.faces[0]);

    // Every swaying face gets its turn over the next frames, the far ones as well as the near.
    const drawn: Set<ILightShadowFace> = new Set();

    for (let frame: number = 0; frame < 3; frame += 1) {
      planner.markDrawn();
      plan(planner, lights, { isDrawn: false, isWindy: true });
      planner.queue.forEach((face: ILightShadowFace) => drawn.add(face));
    }

    expect(drawn.size).toBe(12);
  });

  it("gives a point light six faces looking along the world's axes", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    const [entry] = plan(planner, [createRequest({ isSpot: false })]) as Array<ILightShadowEntry>;
    const looking: Array<Array<number>> = entry.faces.map((face: ILightShadowFace) =>
      new Vector3(0, 0, -1)
        .transformDirection(face.world)
        .toArray()
        .map((it: number) => Math.round(it) + 0)
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

  it("projects a point onto a face where the lights pass looks it up, for every face of a point and a spot", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    const spotBasis: ILightBasis = toLightBasis(
      {
        animatorScale: 0,
        color: [1, 1, 1],
        cone: 1.2,
        direction: [0.3, -1, 0.2],
        isLevel: false,
        isShadowed: true,
        kind: ERendererLightKind.SPOT,
        near: 0.1,
        position: [2, 5, -3],
        range: 10,
        right: [1, 0.2, 0.4],
      },
      createLightBasis()
    );
    const pointBasis: ILightBasis = createLightBasis();
    const [spot, point] = plan(planner, [
      createRequest({ cone: 1.2, direction: spotBasis.direction, position: spotBasis.position, up: spotBasis.up }),
      createRequest({ isSpot: false, position: new Vector3(-4, 2, 7) }),
    ]) as Array<ILightShadowEntry>;

    pointBasis.position.set(-4, 2, 7);

    const probes: Array<[ILightShadowEntry, ILightBasis, boolean, Vector3]> = [
      [
        spot,
        spotBasis,
        true,
        spotBasis.direction
          .clone()
          .multiplyScalar(4)
          .add(new Vector3(0.4, 0.1, -0.3)),
      ],
      ...LIGHT_SHADOW_POINT_FACES.map(({ direction }): [ILightShadowEntry, ILightBasis, boolean, Vector3] => [
        point,
        pointBasis,
        false,
        new Vector3(...direction).multiplyScalar(3).add(new Vector3(0.3, -0.2, 0.25)),
      ]),
    ];

    for (const [entry, basis, isSpot, offset] of probes) {
      const [face, u, v] = toLookedUpUv(isSpot, basis, offset, 1.2);
      const [projectedU, projectedV] = toProjectedUv(entry.faces[face], offset.clone().add(basis.position));

      expect(u).toBeCloseTo(projectedU, 5);
      expect(v).toBeCloseTo(projectedV, 5);
    }
  });

  it("keeps a light given less than it asked as it is, rather than making its faces again every frame", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    // Fifteen 1024 faces and a 512 leave three 512 squares: the last light asks 1024 and is given 512.
    const lights: Array<ILightShadowRequest> = [
      ...Array.from({ length: 15 }, (_, index: number) => createRequest({ distance: index, range: 200 })),
      createRequest({ distance: 11.9 }),
      createRequest({ distance: 20, range: 200 }),
    ];

    function toVersions(entries: Array<Nullable<ILightShadowEntry>>): Array<Nullable<number>> {
      return entries.map((entry: Nullable<ILightShadowEntry>) => entry?.faces[0].version ?? null);
    }

    const first: Array<Nullable<number>> = toVersions(plan(planner, lights, { budget: 17 }));

    expect(planner.getEntry(16)?.size).toBe(512);
    expect(planner.getEntry(16)?.asked).toBe(1024);
    // Given less than it asked: the next frame asks every face for smaller.
    expect(planner.sizeScale).toBeLessThan(1);

    const second: Array<Nullable<number>> = toVersions(plan(planner, lights, { budget: 17 }));

    // Every light whose ask stands keeps its faces, the one given less among them; the 512 now asks for a 256.
    expect(second.filter((_, index: number) => index !== 15)).toEqual(first.filter((_, index: number) => index !== 15));
    expect(second[15]).not.toBe(first[15]);
  });

  it("never gives one light in view another's squares, and asks every face for smaller once they want too much", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    // A 1024 face each: sixteen fill the atlas, and the seventeenth, the farthest, finds no room.
    const lights: Array<ILightShadowRequest> = Array.from({ length: 17 }, (_, index: number) =>
      createRequest({ distance: index, range: 200 })
    );

    const first: Array<Nullable<ILightShadowEntry>> = plan(planner, lights, { budget: 17 });

    expect(first.slice(0, 16).every((it: Nullable<ILightShadowEntry>) => it?.size === 1024)).toBe(true);
    expect(first[16]).toBeNull();
    expect(planner.sizeScale).toBeCloseTo(0.7, 6);

    let entries: Array<Nullable<ILightShadowEntry>> = first;

    for (let frame: number = 0; frame < 8 && !entries.every((it) => it?.size === 512); frame += 1) {
      entries = plan(planner, lights);
    }

    // Small enough for every light to have room, each drawn again in its new square.
    expect(entries.every((it: Nullable<ILightShadowEntry>) => it?.size === 512)).toBe(true);
  });

  it("asks for larger again once the lights in view fit, whatever the lights out of view still hold", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    const crowd: Array<ILightShadowRequest> = Array.from({ length: 40 }, (_, index: number) =>
      createRequest({ distance: index, range: 200 })
    );

    for (let frame: number = 0; frame < 12; frame += 1) {
      plan(planner, crowd, { budget: 40 });
    }

    expect(planner.sizeScale).toBeLessThan(1);

    // Two of them left in view, the rest still holding their squares out of it.
    let pair: Array<Nullable<ILightShadowEntry>> = [];

    for (let frame: number = 0; frame < 40; frame += 1) {
      pair = plan(planner, crowd.slice(0, 2));
    }

    expect(planner.sizeScale).toBe(1);
    expect(pair.map((entry: Nullable<ILightShadowEntry>) => entry?.size)).toEqual([1024, 1024]);
  });

  it("makes room from the light out of view longest, and none from one in view", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    const large: ILightShadowRequest = createRequest({ range: 200 });

    plan(
      planner,
      Array.from({ length: 16 }, () => large),
      { budget: 16 }
    );

    // Lights 1..15 seen again: light 0 is now out of view the longest.
    planner.begin(false);
    Array.from({ length: 15 }, (_, index: number) => planner.request(index + 1, large));
    planner.finish(16);
    planner.markDrawn();

    // Light 16 alone in view takes light 0's room, the rest keeping theirs.
    planner.begin(false);
    planner.request(16, large);
    planner.finish(8);

    expect(planner.getEntry(16)?.size).toBe(1024);
    expect(planner.atlas.used).toBe(16 * 1024 * 1024);

    planner.begin(false);
    Array.from({ length: 15 }, (_, index: number) => planner.request(index + 1, large));
    planner.finish(0);

    expect(Array.from({ length: 15 }, (_, index: number) => planner.getEntry(index + 1)?.size)).toEqual(
      Array.from({ length: 15 }, () => 1024)
    );
  });

  it("lights a point only once all six faces are drawn", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    const point: ILightShadowRequest = createRequest({ isSpot: false });

    expect(plan(planner, [point], { budget: 4 })).toEqual([null]);
    expect(plan(planner, [point], { budget: 4 })[0]?.faces).toHaveLength(6);
  });

  it("keeps showing a light's old faces while its new ones for another size are drawn", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    const [old] = plan(planner, [createRequest()]) as Array<ILightShadowEntry>;
    // Far off: it asks for a smaller square, and nothing is drawn this frame.
    const far: ILightShadowRequest = createRequest({ distance: 12 });

    expect(plan(planner, [far], { budget: 0 })).toEqual([old]);

    const [next] = plan(planner, [far]) as Array<ILightShadowEntry>;

    expect(next).not.toBe(old);
    expect(next.size).toBe(512);
    expect(planner.atlas.used).toBe(512 * 512);
  });

  it("draws every face again once the atlas lost what it held, in the squares it had", () => {
    const planner: LightShadowPlanner = new LightShadowPlanner(new StaticShadowChanges());
    const [entry] = plan(planner, [createRequest()]) as Array<ILightShadowEntry>;

    planner.forgetDrawn();

    expect(plan(planner, [createRequest()], { budget: 0 })).toEqual([null]);
    expect(plan(planner, [createRequest()])).toEqual([entry]);
  });
});
