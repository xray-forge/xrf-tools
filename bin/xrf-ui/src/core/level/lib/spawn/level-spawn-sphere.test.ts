import { describe, expect, it } from "@jest/globals";

import { ILevelSpawnModel } from "@/core/level/lib/render/level-render-protocol";
import { ILevelSpawnSphere, toLevelSpawnPosition, toLevelSpawnSphere } from "@/core/level/lib/spawn/level-spawn-sphere";
import { mockLevelSpawnModel, mockLevelSpawnObject } from "@/fixtures/mocks/level.mocks";
import { mockVisualTransform } from "@/fixtures/mocks/visual.mocks";

describe("toLevelSpawnSphere", () => {
  it("stands its visual's declared sphere where the object stands", () => {
    const model: ILevelSpawnModel = mockLevelSpawnModel("crate");

    model.description.description.declaredBounds.boundingSphere = { center: { x: 0, y: 0.5, z: 0 }, radius: 0.75 };

    const sphere: ILevelSpawnSphere = toLevelSpawnSphere(
      mockLevelSpawnObject({ transform: mockVisualTransform({ x: 10, y: 1, z: -4 }) }),
      model
    );

    expect(sphere).toEqual({ center: { x: 10, y: 1.5, z: -4 }, radius: 0.75 });
  });

  it("takes a metre about where it stands while its model is read", () => {
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
