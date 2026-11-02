import { describe, expect, it } from "@jest/globals";

import { clampMotionFps, formatMotionTiming } from "@/core/visuals/lib/visual-motion";

describe("clampMotionFps", () => {
  it("holds a rate to what a ticker can keep", () => {
    expect(clampMotionFps(30)).toBe(30);
    expect(clampMotionFps(0)).toBe(1);
    expect(clampMotionFps(-10)).toBe(1);
    expect(clampMotionFps(500)).toBe(120);
  });
});

describe("formatMotionTiming", () => {
  it("names only the sample rate when the motion plays at it", () => {
    expect(formatMotionTiming(269, 1)).toBe("269 frames at 30 fps");
  });

  it("names the speed that makes a duration disagree with the frame count", () => {
    expect(formatMotionTiming(269, 1.2)).toBe("269 frames at 30 fps, played at speed 1.2");
  });

  it("says nothing about a speed it was not given", () => {
    // A non-finite `f32` crosses as null, which is a speed nothing can be said about rather than a slow motion.
    expect(formatMotionTiming(269, null)).toBe("269 frames at 30 fps");
  });
});
