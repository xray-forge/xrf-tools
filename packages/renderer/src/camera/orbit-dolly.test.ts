import { describe, expect, it } from "@jest/globals";

import { DOLLY_STEP, TDollyPoint, toDolliedPosition } from "#/camera/orbit-dolly";

describe("toDolliedPosition", () => {
  it("moves along the line to what the camera orbits, without turning it", () => {
    const moved: TDollyPoint = toDolliedPosition([0, 0, 10], [0, 0, 0], 1 / DOLLY_STEP);

    expect(moved[2]).toBeCloseTo(10 / DOLLY_STEP);
    expect(moved[0]).toBe(0);
    expect(moved[1]).toBe(0);
  });

  it("steps by a share rather than a length, so a notch feels the same at any distance", () => {
    const near: TDollyPoint = toDolliedPosition([0, 0, 1], [0, 0, 0], DOLLY_STEP);
    const far: TDollyPoint = toDolliedPosition([0, 0, 1000], [0, 0, 0], DOLLY_STEP);

    // A fixed length would leave a pistol untouched and throw an actor out of frame, the two extremes this viewer
    // opens back to back.
    expect(near[2] / 1).toBeCloseTo(far[2] / 1000);
  });

  it("orbits whatever the camera is looking at, not the origin", () => {
    const moved: TDollyPoint = toDolliedPosition([10, 5, 0], [8, 5, 0], 0.5);

    expect(moved).toEqual([9, 5, 0]);
  });

  it("holds at the bounds the controls declare", () => {
    expect(toDolliedPosition([0, 0, 10], [0, 0, 0], 0.1, 4)[2]).toBeCloseTo(4);
    expect(toDolliedPosition([0, 0, 10], [0, 0, 0], 10, 0, 20)[2]).toBeCloseTo(20);
  });

  it("leaves a camera sitting on its target alone", () => {
    // Nothing to move along: every direction is as good as any other, and the ratio would be a division by zero.
    expect(toDolliedPosition([2, 2, 2], [2, 2, 2], DOLLY_STEP)).toEqual([2, 2, 2]);
  });
});
