import { describe, expect, it } from "@jest/globals";

import { formatDuration, formatSeconds } from "@/lib/format/duration";

describe("formatDuration", () => {
  it("keeps sub second runs in milliseconds", () => {
    expect(formatDuration(123)).toBe("123 ms");
    expect(formatDuration(999)).toBe("999 ms");
  });

  it("rounds fractional milliseconds rather than printing them", () => {
    // The result components used to render `duration / 1000` directly, producing `0.1234 sec`.
    expect(formatDuration(123.456)).toBe("123 ms");
  });

  it("switches to seconds with one decimal", () => {
    expect(formatDuration(1000)).toBe("1.0 s");
    expect(formatDuration(59_400)).toBe("59.4 s");
  });

  it("switches to minutes once a run is long enough to care", () => {
    expect(formatDuration(60_000)).toBe("1 m 0 s");
    expect(formatDuration(98_431)).toBe("1 m 38 s");
    expect(formatDuration(185_000)).toBe("3 m 5 s");
    expect(formatDuration(3_599_000)).toBe("59 m 59 s");
  });

  it("drops the seconds once an hour is on the clock, which an uptime reaches and a run rarely does", () => {
    expect(formatDuration(3_600_000)).toBe("1 h 0 m");
    expect(formatDuration(9_000_000)).toBe("2 h 30 m");
    expect(formatDuration(180_000_000)).toBe("50 h 0 m");
  });
});

describe("formatSeconds", () => {
  it("keeps the unit and two decimals rather than switching either", () => {
    expect(formatSeconds(1.53)).toBe("1.53 s");
    expect(formatSeconds(0)).toBe("0.00 s");
    expect(formatSeconds(49.366_667)).toBe("49.37 s");
  });

  it("stays in seconds past a minute, because a measurement is not a run", () => {
    expect(formatSeconds(65)).toBe("65.00 s");
  });

  it("keeps an absent measurement visibly absent", () => {
    // A rust `f32` arrives as `number | null`, and a non-finite one rendered as a number reads as a measured result.
    expect(formatSeconds(null)).toBe("— s");
    expect(formatSeconds(Number.NaN)).toBe("— s");
    expect(formatSeconds(Number.POSITIVE_INFINITY)).toBe("— s");
  });
});
