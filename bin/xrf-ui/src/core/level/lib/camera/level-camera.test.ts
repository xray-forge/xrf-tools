import { describe, expect, it } from "@jest/globals";
import { TRendererVector } from "@xrf/renderer";

import { ILevelCamera } from "./level-camera";
import { toLevelCamera } from "./level-camera-reading";

function placed(position: TRendererVector, target: TRendererVector): ILevelCamera {
  return toLevelCamera({ position, target });
}

describe("toLevelCamera", () => {
  // The packer negates `z`, so a readout taken straight off the camera would disagree with the level's own data on
  // one axis - and only on one, which is exactly the kind of wrong that goes unnoticed.
  it("states the position in the level's own axes", () => {
    expect(placed([10, 20, 30], [10, 20, 0]).position).toEqual({ x: 10, y: 20, z: -30 });
  });

  it("reads a heading in the same axes as the position", () => {
    // Looking along renderer -z, which is the level's +z, and so a heading of zero.
    expect(placed([0, 0, 0], [0, 0, -1]).heading).toBeCloseTo(0);
    // A quarter turn from it, which the engine measures towards -x.
    expect(placed([0, 0, 0], [-1, 0, 0]).heading).toBeCloseTo(Math.PI / 2);
  });

  it("reads pitch positive above the horizon", () => {
    expect(placed([0, 0, 0], [0, 1, -1]).pitch).toBeCloseTo(Math.PI / 4);
    expect(placed([0, 0, 0], [0, -1, -1]).pitch).toBeCloseTo(-Math.PI / 4);
  });

  it("reads where it faces from where it stands, not from the origin", () => {
    expect(placed([5, 0, 5], [5, 0, 4]).heading).toBeCloseTo(0);
  });
});
