import { describe, expect, it } from "@jest/globals";

import { RenderFrameTimer } from "#/frame/render-frame-timer";

describe("RenderFrameTimer", () => {
  // The first frame has nothing to measure against, so reporting one would be inventing it.
  it("reports nothing until it has seen two frames", () => {
    const timer: RenderFrameTimer = new RenderFrameTimer();

    timer.sample(1000);

    expect(timer.frameTime).toBe(0);
  });

  it("averages the frames it has seen", () => {
    const timer: RenderFrameTimer = new RenderFrameTimer();

    timer.sample(0);
    timer.sample(10);
    timer.sample(30);

    expect(timer.frameTime).toBe(15);
  });

  it("forgets the window when the viewport does", () => {
    const timer: RenderFrameTimer = new RenderFrameTimer();

    timer.sample(0);
    timer.sample(10);
    timer.reset();

    expect(timer.frameTime).toBe(0);
  });
});

describe("RenderFrameTimer frame rate", () => {
  it("derives a frame rate from the mean and reports none before a frame has been timed", () => {
    const timer: RenderFrameTimer = new RenderFrameTimer();

    expect(timer.framesPerSecond).toBe(0);

    timer.sample(0);
    timer.sample(20);

    expect(timer.framesPerSecond).toBeCloseTo(50);
  });
});
