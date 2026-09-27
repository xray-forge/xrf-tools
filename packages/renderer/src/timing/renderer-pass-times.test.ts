import { describe, expect, it } from "@jest/globals";

import { TIssuedRender } from "#/timing/renderer-pass-inspector";
import { toFramePassTimes } from "#/timing/renderer-pass-times";

describe("toFramePassTimes", () => {
  it("sums the renders of one pass within a frame", () => {
    const issued: Map<number, Array<TIssuedRender>> = new Map([
      [
        7,
        [
          ["1:f7", "gbuffer"],
          ["2:f7", "gbuffer"],
          ["3:f7", "combine"],
        ],
      ],
    ]);
    const durations: Map<string, number> = new Map([
      ["1:f7", 0.25],
      ["2:f7", 0.5],
      ["3:f7", 0.125],
    ]);

    const { frames, consumed } = toFramePassTimes(issued, (uid) => durations.get(uid));

    expect(frames).toEqual([
      new Map([
        ["gbuffer", 0.75],
        ["combine", 0.125],
      ]),
    ]);
    expect(consumed).toEqual([7]);
  });

  it("keeps frames apart, in the order they were issued", () => {
    const issued: Map<number, Array<TIssuedRender>> = new Map([
      [11, [["4:f11", "clear"]]],
      [12, [["5:f12", "clear"]]],
    ]);
    const durations: Map<string, number> = new Map([
      ["5:f12", 2],
      ["4:f11", 1],
    ]);

    const { frames } = toFramePassTimes(issued, (uid) => durations.get(uid));

    expect(frames).toEqual([new Map([["clear", 1]]), new Map([["clear", 2]])]);
  });

  it("leaves a frame nothing of has resolved for a later read", () => {
    const issued: Map<number, Array<TIssuedRender>> = new Map([
      [3, [["1:f3", "clear"]]],
      [4, [["2:f4", "clear"]]],
    ]);

    const { frames, consumed } = toFramePassTimes(issued, (uid) => (uid === "1:f3" ? 0.5 : undefined));

    expect(frames).toEqual([new Map([["clear", 0.5]])]);
    expect(consumed).toEqual([3]);
  });

  // Three resolves a frame's renders together: one of them left without a duration is one it never timed.
  it("reads a frame that resolved whole, without the renders three never timed", () => {
    const issued: Map<number, Array<TIssuedRender>> = new Map([
      [
        5,
        [
          ["1:f5", "clear"],
          ["2:f5", "combine"],
        ],
      ],
    ]);

    const { frames, consumed } = toFramePassTimes(issued, (uid) => (uid === "1:f5" ? 0.5 : undefined));

    expect(frames).toEqual([new Map([["clear", 0.5]])]);
    expect(consumed).toEqual([5]);
  });

  // A resolved zero is a pass shorter than the quantisation step, which is a reading and not a missing one.
  it("counts a zero duration as resolved", () => {
    const { frames, consumed } = toFramePassTimes(new Map([[1, [["1:f1", "clear"] as TIssuedRender]]]), () => 0);

    expect(frames).toEqual([new Map([["clear", 0]])]);
    expect(consumed).toEqual([1]);
  });
});
