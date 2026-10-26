import { describe, expect, it } from "@jest/globals";

import { formatWaveSpeed } from "@/core/render/lib/features/render-water-choices";

describe("formatWaveSpeed", () => {
  it("reads the wave speed as the whole number it is set in", () => {
    expect(formatWaveSpeed(12)).toBe("12");
  });
});
