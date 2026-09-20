import { describe, expect, it } from "@jest/globals";

import { formatDegrees } from "@/lib/format/angle";

describe("formatDegrees", () => {
  it("writes an angle with the sign every surface writes it with", () => {
    expect(formatDegrees(45)).toBe("45°");
    expect(formatDegrees(-15)).toBe("-15°");
  });

  it("rounds to whole degrees, which is what a person sets", () => {
    expect(formatDegrees(44.6)).toBe("45°");
  });

  it("keeps the decimals it is asked for", () => {
    expect(formatDegrees(44.62, 1)).toBe("44.6°");
  });
});
