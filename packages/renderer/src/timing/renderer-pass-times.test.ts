import { describe, expect, it } from "@jest/globals";

import { toFramePassTimes } from "#/timing/renderer-pass-times";

describe("toFramePassTimes", () => {
  it("sums the renders of one pass within a frame", () => {
    const issued: Map<string, string> = new Map([
      ["1:f7", "gbuffer"],
      ["2:f7", "gbuffer"],
      ["3:f7", "combine"],
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
    expect(consumed).toEqual(["1:f7", "2:f7", "3:f7"]);
  });

  it("keeps frames apart and orders them oldest first", () => {
    const issued: Map<string, string> = new Map([
      ["5:f12", "clear"],
      ["4:f11", "clear"],
    ]);
    const durations: Map<string, number> = new Map([
      ["5:f12", 2],
      ["4:f11", 1],
    ]);

    const { frames } = toFramePassTimes(issued, (uid) => durations.get(uid));

    expect(frames).toEqual([new Map([["clear", 1]]), new Map([["clear", 2]])]);
  });

  it("leaves a render that has not resolved for a later read", () => {
    const issued: Map<string, string> = new Map([
      ["1:f3", "clear"],
      ["2:f4", "clear"],
    ]);

    const { frames, consumed } = toFramePassTimes(issued, (uid) => (uid === "1:f3" ? 0.5 : undefined));

    expect(frames).toEqual([new Map([["clear", 0.5]])]);
    expect(consumed).toEqual(["1:f3"]);
  });

  // A resolved zero is a pass shorter than the quantisation step, which is a reading and not a missing one.
  it("counts a zero duration as resolved", () => {
    const { frames, consumed } = toFramePassTimes(new Map([["1:f1", "clear"]]), () => 0);

    expect(frames).toEqual([new Map([["clear", 0]])]);
    expect(consumed).toEqual(["1:f1"]);
  });

  it("ignores a uid that names no frame", () => {
    const { frames, consumed } = toFramePassTimes(new Map([["unframed", "clear"]]), () => 1);

    expect(frames).toEqual([]);
    expect(consumed).toEqual([]);
  });
});
