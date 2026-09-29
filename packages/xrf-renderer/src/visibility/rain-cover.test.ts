import { describe, expect, it } from "@jest/globals";
import { Vector3, Vector4 } from "three/webgpu";

import { RAIN_COVER_DEPTH, RAIN_COVER_WIDTH, RainCover } from "#/visibility/rain-cover";

/** Whether a point is inside every plane. */
function isInside(planes: ReadonlyArray<Vector4>, point: Vector3): boolean {
  return planes.every((plane: Vector4) => plane.x * point.x + plane.y * point.y + plane.z * point.z + plane.w >= 0);
}

describe("RainCover", () => {
  it("stands a whole step at a time over the camera, and moves only when the camera leaves its step", () => {
    const cover: RainCover = new RainCover();

    expect(cover.fit(new Vector3(1, 0, 1))).toBe(true);
    expect(cover.center.toArray()).toEqual([0, 60, 0]);
    expect(cover.fit(new Vector3(1.9, 1, -1.9))).toBe(false);
    expect(cover.version).toBe(1);
    expect(cover.fit(new Vector3(2.1, 0, 0))).toBe(true);
    expect(cover.center.x).toBe(4);
    expect(cover.version).toBe(2);
  });

  it("culls by the box it sees, from the height it is seen from down as far as it reaches", () => {
    const cover: RainCover = new RainCover();
    const half: number = RAIN_COVER_WIDTH / 2;

    cover.fit(new Vector3(0, 0, 0));

    expect(isInside(cover.planes, new Vector3(half - 0.1, 0, -half + 0.1))).toBe(true);
    expect(isInside(cover.planes, new Vector3(half + 0.1, 0, 0))).toBe(false);
    expect(isInside(cover.planes, new Vector3(0, 61, 0))).toBe(false);
    expect(isInside(cover.planes, new Vector3(0, 60 - RAIN_COVER_DEPTH + 0.1, 0))).toBe(true);
  });

  it("looks straight down, the level's x across it and its z up it the other way", () => {
    const cover: RainCover = new RainCover();

    cover.fit(new Vector3(0, 0, 0));

    const ahead: Vector3 = new Vector3();

    cover.camera.getWorldDirection(ahead);

    expect(ahead.distanceTo(new Vector3(0, -1, 0))).toBeCloseTo(0, 6);
    // A point east of centre projects to the right, one south of it (towards `+z`) to the bottom.
    expect(new Vector3(RAIN_COVER_WIDTH / 4, 0, 0).project(cover.camera).x).toBeCloseTo(0.5, 6);
    expect(new Vector3(0, 0, RAIN_COVER_WIDTH / 4).project(cover.camera).y).toBeCloseTo(-0.5, 6);
  });
});
