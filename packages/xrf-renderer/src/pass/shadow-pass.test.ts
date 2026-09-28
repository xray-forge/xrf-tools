import { describe, expect, it } from "@jest/globals";

import { isShadowCascadeDue } from "#/pass/shadow-pass";

describe("isShadowCascadeDue", () => {
  it("draws the nearest cascade every frame, the second every other, the third every fourth", () => {
    const frames: Array<number> = Array.from({ length: 8 }, (_, index: number) => index + 1);

    expect(frames.filter((frame) => isShadowCascadeDue(0, frame))).toHaveLength(8);
    expect(frames.filter((frame) => isShadowCascadeDue(1, frame))).toEqual([1, 3, 5, 7]);
    expect(frames.filter((frame) => isShadowCascadeDue(2, frame))).toEqual([2, 6]);
    expect(frames.filter((frame) => isShadowCascadeDue(3, frame))).toEqual([4]);
  });

  it("never draws two far cascades in one frame", () => {
    for (let frame = 1; frame <= 64; frame += 1) {
      expect([1, 2, 3].filter((view) => isShadowCascadeDue(view, frame)).length).toBeLessThanOrEqual(1);
    }
  });
});
