import { describe, expect, it } from "@jest/globals";

import { IRendererGeometry } from "#/contract/scene/renderer-geometry";
import { IRendererGeometryGroup } from "#/contract/scene/renderer-geometry-group";
import { createRendererBox } from "#/geometry/renderer-box-geometry";

describe("createRendererBox", () => {
  it("draws each face with its own slot, in three's face order", () => {
    const box: IRendererGeometry = createRendererBox(2, 2, 2);
    const front: number = box.index![box.groups[4].start];

    expect(box.groups.map((group: IRendererGeometryGroup) => group.slot)).toEqual([0, 1, 2, 3, 4, 5]);
    // The +z face faces +z.
    expect([box.normal![front * 3], box.normal![front * 3 + 1], box.normal![front * 3 + 2]]).toEqual([0, 0, 1]);
  });

  it("keeps the depth it was given, so a slab turned away shows an edge", () => {
    const box: IRendererGeometry = createRendererBox(2, 2, 0.04);
    const depths: Array<number> = Array.from(
      { length: box.position.length / 3 },
      (_: unknown, at: number) => box.position[at * 3 + 2]
    );

    expect(Math.max(...depths) - Math.min(...depths)).toBeCloseTo(0.04);
  });
});
