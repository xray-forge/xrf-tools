import { describe, expect, it } from "@jest/globals";

import { decodeXrayBumpTexel, IMaterialBumpTexel } from "@/core/materials/lib/material-bump-texel";

describe("decodeXrayBumpTexel", () => {
  it("reconstructs the normal, gloss and height of a synthesized texel pair as sload.h does", () => {
    // Nu.wzy is (1.0, 0.5, 0.5) and the companion's error term is (-0.5, -0.5, 0.0), so the normal leans along x and z.
    // Height is the companion's fourth channel, not its third: the third is the error of the normal's z, which is
    // already spent correcting it above.
    const decoded: IMaterialBumpTexel = decodeXrayBumpTexel([0.6, 0.5, 0.5, 1.0], [0.5, 0.5, 1.0, 0.25]);

    expect(decoded.normal[0]).toBeCloseTo(0.5);
    expect(decoded.normal[1]).toBeCloseTo(0.0);
    expect(decoded.normal[2]).toBeCloseTo(0.5);
    expect(decoded.gloss).toBeCloseTo(0.36);
    expect(decoded.height).toBeCloseTo(0.25);
  });

  it("decodes the flat dummy pair to a normal pointing straight out", () => {
    // What the engine draws for a bump it cannot find: no tilt, no gloss to speak of.
    const decoded: IMaterialBumpTexel = decodeXrayBumpTexel([0.0, 0.5, 0.5, 0.5], [0.5, 0.5, 0.5, 0.5]);

    expect(decoded.normal).toEqual([0, 0, 0]);
    expect(decoded.gloss).toBe(0);
  });
});
