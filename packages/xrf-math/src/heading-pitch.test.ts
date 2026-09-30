import { describe, expect, it } from "@jest/globals";

import { toDirection, toHeadingPitch } from "#/heading-pitch";

describe("toDirection", () => {
  it("points along +z at no heading, towards -x a quarter turn on, and up at a quarter pitch", () => {
    const [x, y, z] = toDirection({ heading: Math.PI / 2, pitch: 0 });

    expect(toDirection({ heading: 0, pitch: 0 })).toEqual([-0, 0, 1]);
    expect([x, y, z].map((it: number) => Number(it.toFixed(6)))).toEqual([-1, 0, 0]);
    expect(toDirection({ heading: 0, pitch: Math.PI / 2 })[1]).toBe(1);
  });
});

describe("toHeadingPitch", () => {
  it("reads back the angles a direction was made from", () => {
    const read = toHeadingPitch(toDirection({ heading: -2, pitch: 0.4 }));

    expect(read.heading).toBeCloseTo(-2, 10);
    expect(read.pitch).toBeCloseTo(0.4, 10);
  });

  // `getHP` takes `fis_zero` of both across components, which compares against `EPS_S`.
  it("reads no heading for a direction straight up or down, of any length", () => {
    expect(toHeadingPitch([0, 5, 0])).toEqual({ heading: 0, pitch: Math.PI / 2 });
    expect(toHeadingPitch([0, -1, 0])).toEqual({ heading: 0, pitch: -Math.PI / 2 });
    expect(toHeadingPitch([0.00000001, 1, 0]).heading).toBe(0);
  });
});
