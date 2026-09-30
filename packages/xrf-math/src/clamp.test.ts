import { describe, expect, it } from "@jest/globals";

import { clamp, saturate } from "#/clamp";

describe("clamp", () => {
  it("keeps a value within its bounds and moves one outside them to the bound it passed", () => {
    expect(clamp(5, 0, 10)).toBe(5);
    expect(clamp(-5, 0, 10)).toBe(0);
    expect(clamp(15, 0, 10)).toBe(10);
  });

  // An empty list's last index is below its first, and every caller that clamps into one relies on this.
  it("takes the upper bound when the bounds cross", () => {
    expect(clamp(0, 0, -1)).toBe(-1);
  });
});

describe("saturate", () => {
  it("holds a value within zero and one", () => {
    expect(saturate(0.25)).toBe(0.25);
    expect(saturate(-1)).toBe(0);
    expect(saturate(2)).toBe(1);
  });
});
