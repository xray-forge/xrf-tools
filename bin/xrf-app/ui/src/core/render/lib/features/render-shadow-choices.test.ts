import { describe, expect, it } from "@jest/globals";

import { formatShadowBias } from "@/core/render/lib/features/render-shadow-choices";

describe("formatShadowBias", () => {
  it("reads the bias to the quarter step it is set in", () => {
    expect(formatShadowBias(0.25)).toBe("0.25 texels");
  });
});
