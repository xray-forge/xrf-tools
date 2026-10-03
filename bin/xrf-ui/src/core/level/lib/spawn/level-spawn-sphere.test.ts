import { describe, expect, it } from "@jest/globals";

import { ILevelSpawnSphere, toLevelSpawnPosition, toLevelSpawnSphere } from "@/core/level/lib/spawn/level-spawn-sphere";
import { mockLevelSpawnObject } from "@/fixtures/mocks/level.mocks";
import { mockVisualTransform } from "@/fixtures/mocks/visual.mocks";

describe("toLevelSpawnSphere", () => {
  it("takes the sphere the renderer holds for its model", () => {
    const sphere: ILevelSpawnSphere = toLevelSpawnSphere(
      mockLevelSpawnObject({ transform: mockVisualTransform({ x: 10, y: 1, z: -4 }) }),
      [10, 1.5, -4, 0.75]
    );

    expect(sphere).toEqual({ center: { x: 10, y: 1.5, z: -4 }, radius: 0.75 });
  });

  it("takes a metre about where it stands before its model is drawn", () => {
    expect(
      toLevelSpawnSphere(mockLevelSpawnObject({ transform: mockVisualTransform({ x: 1, y: 2, z: 3 }) }), null)
    ).toEqual({
      center: { x: 1, y: 2, z: 3 },
      radius: 1,
    });
  });
});

describe("toLevelSpawnPosition", () => {
  it("states where an object stands as the engine does, z the other way", () => {
    expect(toLevelSpawnPosition(mockVisualTransform({ x: 1, y: 2, z: -3 }))).toEqual({ x: 1, y: 2, z: 3 });
  });
});
