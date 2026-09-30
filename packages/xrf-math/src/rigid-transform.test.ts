import { describe, expect, it } from "@jest/globals";

import { composeRigidTransforms, invertRigidTransform, rotateByRigidTransform } from "#/rigid-transform";

/** A quarter turn about `+y`, taking `+x` to `-z`, then a step of `(1, 2, 3)`. */
const QUARTER_TURN_STEP: ReadonlyArray<number> = [0, 0, -1, 0, 1, 0, 1, 0, 0, 1, 2, 3];

/** A point through the whole transform: its basis, then its translation. */
function toMoved(transform: ArrayLike<number>, x: number, y: number, z: number): Array<number> {
  const out: Float32Array = new Float32Array(3);

  rotateByRigidTransform(transform, x, y, z, out, 0);

  return [out[0] + transform[9], out[1] + transform[10], out[2] + transform[11]];
}

describe("rotateByRigidTransform", () => {
  it("turns a direction through the basis alone, at the offset asked", () => {
    const out: Float32Array = new Float32Array(5);

    rotateByRigidTransform(QUARTER_TURN_STEP, 1, 0, 0, out, 2);

    expect(Array.from(out)).toEqual([0, 0, 0, 0, -1]);
  });
});

describe("invertRigidTransform", () => {
  it("takes a moved point back where it started", () => {
    const moved: Array<number> = toMoved(QUARTER_TURN_STEP, 4, 5, 6);

    expect(toMoved(invertRigidTransform(QUARTER_TURN_STEP), moved[0], moved[1], moved[2])).toEqual([4, 5, 6]);
  });
});

describe("composeRigidTransforms", () => {
  it("moves a point through the inner transform, then the outer", () => {
    const step: ReadonlyArray<number> = [1, 0, 0, 0, 1, 0, 0, 0, 1, 10, 0, 0];
    const [x, y, z] = toMoved(QUARTER_TURN_STEP, 1, 1, 1);

    expect(toMoved(composeRigidTransforms(step, QUARTER_TURN_STEP), 1, 1, 1)).toEqual(toMoved(step, x, y, z));
  });

  it("is nothing at all with its own inverse", () => {
    const identity: Float32Array = composeRigidTransforms(invertRigidTransform(QUARTER_TURN_STEP), QUARTER_TURN_STEP);

    expect(Array.from(identity)).toEqual([1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0]);
  });
});
