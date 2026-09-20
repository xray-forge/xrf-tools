import { describe, expect, it } from "@jest/globals";

import {
  DEFAULT_FRAME_RATE_LIMIT,
  shouldDrawFrame,
  toFrameInterval,
  toFrameRateLimit,
} from "@/core/render/lib/frame/render-frame-limit";

describe("toFrameRateLimit", () => {
  it("takes a limit this build offers", () => {
    expect(toFrameRateLimit("30")).toBe("30");
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
  it("waits a fraction under the interval asked for", () => {
    expect(toFrameInterval("60")).toBeCloseTo(15);
    expect(toFrameInterval("30")).toBeCloseTo(30);
    expect(toFrameInterval("120")).toBeCloseTo(7.5);
  });

  it("waits not at all for a viewport allowed every frame", () => {
    expect(toFrameInterval("unlimited")).toBe(0);
  });
});

describe("shouldDrawFrame", () => {
  it("draws the first frame, which has nothing to wait behind", () => {
    expect(shouldDrawFrame(0, null, "30")).toBe(true);
  });

  it("drops a frame that arrives inside the budget", () => {
    expect(shouldDrawFrame(1005, 1000, "60")).toBe(false);
  });

  it("draws one that arrives past it", () => {
    expect(shouldDrawFrame(1020, 1000, "60")).toBe(true);
  });

  // The case the allowance is for: a 60Hz display wakes every 16.67ms, but its wakes drift, and a wake that lands a
  // fraction short of 16.67 would be dropped - halving the sixty that was asked for to thirty.
  it("draws a frame that lands just short of the interval", () => {
    expect(shouldDrawFrame(1016.4, 1000, "60")).toBe(true);
  });

  it("draws every frame when nothing is limiting them", () => {
    expect(shouldDrawFrame(1000.1, 1000, "unlimited")).toBe(true);
  });
});
