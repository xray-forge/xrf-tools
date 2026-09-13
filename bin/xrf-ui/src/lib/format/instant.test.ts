import { describe, expect, it } from "@jest/globals";

import { formatInstant } from "@/lib/format/instant";

describe("formatInstant", () => {
  it("states only the clock while the moment is still today's", () => {
    expect(formatInstant(new Date(2026, 8, 13, 9, 4, 5).getTime(), new Date(2026, 8, 13, 18, 0, 0).getTime())).toBe(
      "09:04:05"
    );
  });

  it("spells the date once the moment falls on another day", () => {
    expect(formatInstant(new Date(2026, 8, 11, 23, 58, 1).getTime(), new Date(2026, 8, 13, 0, 2, 0).getTime())).toBe(
      "11 Sep 23:58:01"
    );
  });
});
