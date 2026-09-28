import { describe, expect, it } from "@jest/globals";

import { DEFAULT_FRAME_RATE_LIMIT, toFrameInterval, toFrameRateLimit } from "#/frame/frame-rate-limit";

describe("toFrameRateLimit", () => {
  it("takes a limit this build offers", () => {
    expect(toFrameRateLimit("30")).toBe("30");
    expect(toFrameRateLimit("160")).toBe("160");
    expect(toFrameRateLimit("unlimited")).toBe("unlimited");
  });

  // Written by an older build, a hand edit, or nothing at all: a viewport asked to wait `1000 / NaN` would never
  // draw again.
  it("refuses anything else rather than handing it to a viewport", () => {
    expect(toFrameRateLimit("144")).toBe(DEFAULT_FRAME_RATE_LIMIT);
    expect(toFrameRateLimit(60)).toBe(DEFAULT_FRAME_RATE_LIMIT);
    expect(toFrameRateLimit(null)).toBe(DEFAULT_FRAME_RATE_LIMIT);
    expect(toFrameRateLimit(undefined)).toBe(DEFAULT_FRAME_RATE_LIMIT);
  });
});

describe("toFrameInterval", () => {
  it("waits exactly the interval asked for", () => {
    expect(toFrameInterval("60")).toBeCloseTo(1000 / 60);
    expect(toFrameInterval("30")).toBeCloseTo(1000 / 30);
    expect(toFrameInterval("120")).toBeCloseTo(1000 / 120);
    expect(toFrameInterval("160")).toBeCloseTo(1000 / 160);
  });

  it("waits not at all for a viewport allowed every frame", () => {
    expect(toFrameInterval("unlimited")).toBe(0);
  });
});
