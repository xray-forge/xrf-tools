import { describe, expect, it } from "@jest/globals";

import { formatContactShadowLights } from "@/core/render/lib/features/render-contact-shadow-choices";

describe("formatContactShadowLights", () => {
  it("names the sun alone, and counts the lights past it", () => {
    expect(formatContactShadowLights(0)).toBe("Sun only");
    expect(formatContactShadowLights(1)).toBe("1 light");
    expect(formatContactShadowLights(4)).toBe("4 lights");
  });
});
