import { describe, expect, it } from "@jest/globals";

import { toneMapReinhard } from "#/lighting/tonemap";

describe("toneMapReinhard", () => {
  it("applies the engine's curve with a white point of 1.7", () => {
    // x * (1 + x / 2.89) / (x + 1)
    expect(toneMapReinhard(1, 1)).toBeCloseTo(0.67301, 5);
    expect(toneMapReinhard(0.5, 1)).toBeCloseTo(0.391003, 5);
  });

  it("scales before the curve, not after", () => {
    expect(toneMapReinhard(0.5, 2)).toBeCloseTo(toneMapReinhard(1, 1), 10);
  });

  it("reaches one at the white point itself", () => {
    // x (1 + x / w^2) / (x + 1) = 1 solves to x = w.
    expect(toneMapReinhard(1.7, 1)).toBeCloseTo(1, 10);
  });
});
