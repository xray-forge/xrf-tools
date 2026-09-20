import { describe, expect, it } from "@jest/globals";
import { Box3, Vector3 } from "three";

import { toBoxFloor, toBoxReach, toLevelBox, toOriginReach } from "@/core/level/lib/extent";
import { mockVisualBounds } from "@/fixtures/mocks/visual.mocks";

function boxOf(min: [number, number, number], max: [number, number, number]): Box3 {
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
    const box: Box3 = boxOf([-2, 0, -3], [4, 6, 9]);

    expect(box.min.toArray()).toEqual([-2, 0, -3]);
    expect(box.max.toArray()).toEqual([4, 6, 9]);
  });

  // A level that reports no bounds is ordinary, and everything here answers for one.
  it("is empty for a level that measured nothing", () => {
    expect(toLevelBox(null).isEmpty()).toBe(true);
  });
});

describe("toBoxReach", () => {
  it("reaches the longer horizontal half, ignoring height", () => {
    expect(toBoxReach(boxOf([-10, 0, -2], [10, 900, 2]))).toBe(10);
  });

  it("still reaches for an empty box, so what is built from one is not a point", () => {
    expect(toBoxReach(new Box3())).toBeGreaterThan(0);
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
    expect(toBoxFloor(boxOf([-4, 5, -8], [6, 9, 12]))).toEqual(new Vector3(1, 5, 2));
  });

  it("is the origin for an empty box", () => {
    expect(toBoxFloor(new Box3())).toEqual(new Vector3());
  });
});
