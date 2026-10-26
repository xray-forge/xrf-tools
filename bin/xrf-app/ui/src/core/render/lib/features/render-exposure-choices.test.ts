import { describe, expect, it } from "@jest/globals";

import { formatLowLuminance } from "@/core/render/lib/features/render-exposure-choices";

describe("formatLowLuminance", () => {
  it("keeps the four decimals a luminance floor is set in", () => {
    expect(formatLowLuminance(0.0001)).toBe("0.0001");
  });
});
