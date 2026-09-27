import { describe, expect, it } from "@jest/globals";

import { TFrameRateLimit } from "#/frame/render-frame-limit";
import { RenderFrameLimiter } from "#/frame/render-frame-limiter";

/**
 * @param hertz - The display's rate.
 * @param limit - The rate asked for.
 * @returns Frames drawn over ten seconds of the display's wakes.
 */
function countDrawn(hertz: number, limit: TFrameRateLimit): number {
  const limiter: RenderFrameLimiter = new RenderFrameLimiter();
  let drawn: number = 0;

  for (let wake: number = 0; wake < hertz * 10; wake += 1) {
    if (limiter.take((wake * 1000) / hertz, limit)) {
      drawn += 1;
    }
  }

  return drawn;
}

describe("RenderFrameLimiter", () => {
  it("draws the rate asked for on a display whose wakes do not divide it", () => {
    expect(countDrawn(160, "60")).toBeGreaterThanOrEqual(599);
    expect(countDrawn(160, "60")).toBeLessThanOrEqual(601);
    expect(countDrawn(144, "120")).toBeGreaterThanOrEqual(1199);
    expect(countDrawn(144, "120")).toBeLessThanOrEqual(1201);
    expect(countDrawn(165, "30")).toBeGreaterThanOrEqual(299);
    expect(countDrawn(165, "30")).toBeLessThanOrEqual(301);
    expect(countDrawn(165, "160")).toBeGreaterThanOrEqual(1599);
    expect(countDrawn(165, "160")).toBeLessThanOrEqual(1601);
  });

  it("draws every wake of a display slower than the limit, and every wake without one", () => {
    expect(countDrawn(60, "120")).toBe(600);
    expect(countDrawn(160, "160")).toBe(1600);
    expect(countDrawn(160, "unlimited")).toBe(1600);
  });

  it("draws a wake landing a fraction short of due, as a display's wakes wander", () => {
    const limiter: RenderFrameLimiter = new RenderFrameLimiter();

    expect(limiter.take(0, "60")).toBe(true);
    expect(limiter.take(16.4, "60")).toBe(true);
    expect(limiter.take(20, "60")).toBe(false);
  });

  it("starts afresh after a stall rather than drawing the frames it missed back to back", () => {
    const limiter: RenderFrameLimiter = new RenderFrameLimiter();

    limiter.take(0, "60");

    expect(limiter.take(500, "60")).toBe(true);
    expect(limiter.take(506.25, "60")).toBe(false);
  });
});
