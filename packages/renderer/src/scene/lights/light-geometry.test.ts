import { describe, expect, it } from "@jest/globals";
import { Sphere, Vector3 } from "three/webgpu";

import { ERendererLightKind, IRendererPointLight, IRendererSpotLight } from "#/contract/scene/renderer-lights";
import {
  createLightBasis,
  ILightBasis,
  toLightBasis,
  toLightBound,
  toLightIntensity,
  toLightLod,
  toLightSpatialSphere,
} from "#/scene/lights/light-geometry";

const POINT: IRendererPointLight = {
  animatorScale: 0,
  color: [1, 1, 1],
  isLevel: false,
  isShadowed: false,
  kind: ERendererLightKind.POINT,
  near: 0,
  position: [1, 2, 3],
  range: 4,
};

/** A spot down `-z` from the origin, `range` 10, with the cone given. */
function createSpot(cone: number, part: Partial<IRendererSpotLight> = {}): IRendererSpotLight {
  return {
    ...POINT,
    cone,
    direction: [0, 0, -1],
    kind: ERendererLightKind.SPOT,
    position: [0, 0, 0],
    range: 10,
    right: [1, 0, 0],
    ...part,
  };
}

/** Whether every point of a spot's reach stands within a sphere: its apex, its rim, and the tip of its cap. */
function holdsSpot(sphere: Sphere, spot: IRendererSpotLight): boolean {
  const half: number = spot.cone / 2;
  const reach: number = spot.range + (spot.rangeJitter ?? 0);
  const points: Array<Vector3> = [
    new Vector3(0, 0, 0),
    new Vector3(reach * Math.sin(half), 0, -reach * Math.cos(half)),
    new Vector3(0, 0, -reach),
  ];

  return points.every((point: Vector3) => sphere.center.distanceTo(point) <= sphere.radius + 1e-9);
}

describe("toLightBound", () => {
  it("bounds a point by its range, as far as its range strays", () => {
    const bound: Sphere = toLightBound({ ...POINT, rangeJitter: 1 }, new Sphere());

    expect(bound.center.toArray()).toEqual([1, 2, 3]);
    expect(bound.radius).toBe(5);
  });

  it("bounds a narrow spot through its apex and rim, a wide one by its rim, meeting at a quarter turn", () => {
    const narrow: Sphere = toLightBound(createSpot(Math.PI / 4), new Sphere());
    const wide: Sphere = toLightBound(createSpot(Math.PI * 0.8), new Sphere());
    const [below, above] = [Math.PI / 2 - 1e-6, Math.PI / 2 + 1e-6].map((cone: number) =>
      toLightBound(createSpot(cone), new Sphere())
    );

    expect(narrow.radius).toBeCloseTo(10 / (2 * Math.cos(Math.PI / 8)), 6);
    expect(narrow.center.z).toBeCloseTo(-narrow.radius, 6);
    expect(wide.radius).toBeCloseTo(10 * Math.sin(Math.PI * 0.4), 6);
    expect(below.radius).toBeCloseTo(10 / Math.SQRT2, 4);
    expect(above.radius).toBeCloseTo(below.radius, 4);
    expect(above.center.z).toBeCloseTo(below.center.z, 4);

    for (const cone of [0.2, Math.PI / 4, Math.PI / 2, 2.5]) {
      expect(holdsSpot(toLightBound(createSpot(cone), new Sphere()), createSpot(cone))).toBe(true);
    }

    expect(
      holdsSpot(toLightBound(createSpot(0.6, { rangeJitter: 2 }), new Sphere()), createSpot(0.6, { rangeJitter: 2 }))
    ).toBe(true);
  });
});

describe("toLightSpatialSphere", () => {
  it("takes `light::spatial_move`'s sphere: a narrow spot's `R / (2 cos² h)` ahead, a wide one's `R tan h` at its range", () => {
    const narrow: Sphere = toLightSpatialSphere(createSpot(Math.PI / 3), new Sphere());
    const wide: Sphere = toLightSpatialSphere(createSpot((2 * Math.PI) / 3), new Sphere());

    expect(narrow.radius).toBeCloseTo(10 / (2 * Math.cos(Math.PI / 6) ** 2), 6);
    expect(narrow.center.z).toBeCloseTo(-narrow.radius, 6);
    expect(wide.radius).toBeCloseTo(10 * Math.tan(Math.PI / 3), 6);
    expect(wide.center.z).toBeCloseTo(-10, 6);
    expect(toLightSpatialSphere(POINT, new Sphere()).radius).toBe(4);
  });
});

describe("toLightBasis", () => {
  it("makes the given right square to the direction, the up turned for the mirrored space", () => {
    const basis: ILightBasis = toLightBasis(
      createSpot(1, { direction: [0, -1, 0], right: [1, 0.5, 0] }),
      createLightBasis()
    );

    expect(basis.right.x).toBeCloseTo(1, 6);
    expect(basis.right.y).toBeCloseTo(0, 6);
    expect(basis.up.z).toBeCloseTo(-1, 6);
  });

  it("takes the world's up where a spot gives no right, and the world's forward where it points up", () => {
    const level: ILightBasis = toLightBasis(createSpot(1, { right: [0, 0, 0] }), createLightBasis());
    const down: ILightBasis = toLightBasis(
      createSpot(1, { direction: [0, -1, 0], right: [0, 0, 0] }),
      createLightBasis()
    );

    for (const basis of [level, down]) {
      expect(basis.right.length()).toBeCloseTo(1, 6);
      expect(basis.up.length()).toBeCloseTo(1, 6);
      expect(basis.right.dot(basis.direction)).toBeCloseTo(0, 6);
      expect(basis.up.dot(basis.direction)).toBeCloseTo(0, 6);
    }

    expect(level.up.y).toBeCloseTo(1, 6);
    expect(Math.abs(down.up.z)).toBeCloseTo(1, 6);
  });
});

describe("toLightLod", () => {
  it("fades by `get_LOD`: the square root of the sphere's share between the thresholds", () => {
    const sphere: Sphere = new Sphere(new Vector3(0, 0, -10), 2);
    const area: number = (0.5 * 2) / (100 + 0.00001);

    expect(toLightLod(sphere, new Vector3(), 0.02, 0.001)).toBeCloseTo(Math.sqrt((area - 0.001) / (0.02 - 0.001)), 6);
    expect(toLightLod(sphere, new Vector3(), 0.002, 0.001)).toBe(1);
    expect(toLightLod(sphere, new Vector3(), 0.5, 0.1)).toBe(0);
  });
});

describe("toLightIntensity", () => {
  it("averages the colour's mean and its luminance", () => {
    expect(toLightIntensity([1, 1, 1])).toBeCloseTo(1, 6);
    expect(toLightIntensity([0, 1, 0])).toBeCloseTo((1 / 3 + 0.7154) / 2, 6);
  });
});
