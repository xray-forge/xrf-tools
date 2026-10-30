import { describe, expect, it } from "@jest/globals";

import { mockLevelObject, mockLevelVolumeObject } from "@/core/level/fixtures/level.mocks";
import {
  ILevelObjectSphere,
  toLevelObjectPosition,
  toLevelObjectSphere,
} from "@/core/level/lib/objects/level-object-sphere";
import { mockVisualTransform } from "@/core/visuals/fixtures/visual.mocks";

describe("toLevelObjectSphere", () => {
  it("takes the sphere the renderer holds for its model", () => {
    const sphere: ILevelObjectSphere = toLevelObjectSphere(
      mockLevelObject({ transform: mockVisualTransform({ x: 10, y: 1, z: -4 }) }),
      [10, 1.5, -4, 0.75]
    );

    expect(sphere).toEqual({ center: { x: 10, y: 1.5, z: -4 }, radius: 0.75 });
  });

  it("takes a metre about where it stands before its model is drawn", () => {
    expect(
      toLevelObjectSphere(mockLevelObject({ transform: mockVisualTransform({ x: 1, y: 2, z: 3 }) }), null)
    ).toEqual({
      center: { x: 1, y: 2, z: 3 },
      radius: 1,
    });
  });

  // A sphere's box is its own cube, so the sphere about that box reaches the cube's corners.
  it("measures a volume by the box about its shapes where it stands", () => {
    const sphere: ILevelObjectSphere = toLevelObjectSphere(
      { ...mockLevelVolumeObject(0, 2), transform: mockVisualTransform({ x: 5, y: 0, z: 1 }) },
      null
    );

    expect(sphere.center).toEqual({ x: 5, y: 0, z: 1 });
    expect(sphere.radius).toBeCloseTo(Math.sqrt(12));
  });
});

describe("toLevelObjectPosition", () => {
  it("states where an object stands as the engine does, z the other way", () => {
    expect(toLevelObjectPosition(mockVisualTransform({ x: 1, y: 2, z: -3 }))).toEqual({ x: 1, y: 2, z: 3 });
  });
});
