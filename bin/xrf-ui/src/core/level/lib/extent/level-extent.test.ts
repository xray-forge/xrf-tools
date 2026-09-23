import { describe, expect, it } from "@jest/globals";

import { ILevelBox, toBoxFloor, toBoxReach, toLevelBox, toOriginReach } from "@/core/level/lib/extent";
import { mockVisualBounds } from "@/fixtures/mocks/visual.mocks";

function boxOf(min: [number, number, number], max: [number, number, number]): ILevelBox {
  return toLevelBox(
    mockVisualBounds({
      boundingBox: {
        max: { x: max[0], y: max[1], z: max[2] },
        min: { x: min[0], y: min[1], z: min[2] },
      },
    })
  );
}

describe("toLevelBox", () => {
  it("takes the corners the backend measured", () => {
    const box: ILevelBox = boxOf([-2, 0, -3], [4, 6, 9]);

    expect(box.min).toEqual({ x: -2, y: 0, z: -3 });
    expect(box.max).toEqual({ x: 4, y: 6, z: 9 });
    expect(box.isEmpty).toBe(false);
  });

  // A level that reports no bounds is ordinary, and everything here answers for one.
  it("is empty for a level that measured nothing", () => {
    expect(toLevelBox(null).isEmpty).toBe(true);
  });
});

describe("toBoxReach", () => {
  it("reaches the longer horizontal half, ignoring height", () => {
    expect(toBoxReach(boxOf([-10, 0, -2], [10, 900, 2]))).toBe(10);
  });

  it("still reaches for an empty box, so what is built from one is not a point", () => {
    expect(toBoxReach(toLevelBox(null))).toBeGreaterThan(0);
  });
});

describe("toOriginReach", () => {
  // Not the same question: a level lying a kilometre off the origin needs something that reaches the level, not
  // something the size of it.
  it("reaches the furthest corner from zero rather than across the box", () => {
    expect(toOriginReach(boxOf([900, 0, 900], [1000, 10, 1000]))).toBe(1000);
    expect(toBoxReach(boxOf([900, 0, 900], [1000, 10, 1000]))).toBe(50);
  });
});

describe("toBoxFloor", () => {
  it("is the middle of the box at its lowest point", () => {
    expect(toBoxFloor(boxOf([-4, 5, -8], [6, 9, 12]))).toEqual({ x: 1, y: 5, z: 2 });
  });

  it("is the origin for an empty box", () => {
    expect(toBoxFloor(toLevelBox(null))).toEqual({ x: 0, y: 0, z: 0 });
  });
});
