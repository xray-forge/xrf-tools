import { describe, expect, it } from "@jest/globals";

import { mix } from "#/mix";

describe("mix", () => {
  it("is each end at none and all of the way, and halfway between at a half", () => {
    expect(mix(2, 6, 0)).toBe(2);
    expect(mix(2, 6, 1)).toBe(6);
    expect(mix(2, 6, 0.5)).toBe(4);
  });

  // A zoom about an anchor scales past the anchor's own reach.
  it("carries on past either end", () => {
    expect(mix(2, 6, 2)).toBe(10);
    expect(mix(2, 6, -1)).toBe(-2);
  });
});
