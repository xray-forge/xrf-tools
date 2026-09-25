import { describe, expect, it } from "@jest/globals";

import { toSharpenStrength } from "#/uniforms/sharpen-uniforms";

describe("toSharpenStrength", () => {
  it("remaps a sharpness to RCAS's stops as FSR 2 does: two stops at none, none at the most", () => {
    expect(toSharpenStrength(0)).toBeCloseTo(0.25, 10);
    expect(toSharpenStrength(0.5)).toBeCloseTo(0.5, 10);
    expect(toSharpenStrength(1)).toBe(1);
    expect(toSharpenStrength(3)).toBe(1);
  });
});
