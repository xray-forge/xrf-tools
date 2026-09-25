import { describe, expect, it } from "@jest/globals";

import { IRendererLightAnimator } from "#/contract/scene/renderer-lights";
import { toAnimatedColor, toInterpolatedColor } from "#/lighting/light-animator";

const ANIMATOR: IRendererLightAnimator = {
  colors: [
    [0, 0, 0],
    [200, 100, 50],
    [100, 100, 100],
  ],
  fps: 10,
  frameCount: 20,
  frames: [0, 10, 15],
};

describe("toInterpolatedColor", () => {
  it("holds a key's colour on its frame and blends between the keys either side", () => {
    expect(toInterpolatedColor(ANIMATOR, 10, [])).toEqual([200, 100, 50]);
    expect(toInterpolatedColor(ANIMATOR, 5, [])).toEqual([100, 50, 25]);
    expect(toInterpolatedColor(ANIMATOR, 12, [])).toEqual([160, 100, 70]);
  });

  it("holds the last key's colour past it", () => {
    expect(toInterpolatedColor(ANIMATOR, 19, [])).toEqual([100, 100, 100]);
  });
});

describe("toAnimatedColor", () => {
  // Twenty frames at ten a second: two seconds a loop, stepped to whole frames.
  it("steps the time to the animation's frames and loops it over its length", () => {
    expect(toAnimatedColor(ANIMATOR, 0.55, [])).toEqual([100, 50, 25]);
    expect(toAnimatedColor(ANIMATOR, 2.55, [])).toEqual([100, 50, 25]);
    expect(toAnimatedColor(ANIMATOR, 1.0, [])).toEqual([200, 100, 50]);
  });
});
