import { describe, expect, it } from "@jest/globals";

import { RendererPassTimer } from "#/timing/renderer-pass-timer";

describe("RendererPassTimer", () => {
  it("reports each pass's mean in the order the frame runs them", () => {
    const timer: RendererPassTimer = new RendererPassTimer();

    timer.record(new Map([["combine", 1]]));
    timer.record(
      new Map([
        ["combine", 3],
        ["gbuffer", 4],
      ])
    );

    expect(timer.describe(["gbuffer", "combine"])).toEqual([
      { gpuTime: 4, name: "gbuffer" },
      { gpuTime: 2, name: "combine" },
    ]);
  });

  it("counts a frame a pass issued nothing in as costing it nothing", () => {
    const timer: RendererPassTimer = new RendererPassTimer();

    timer.record(
      new Map([
        ["gbuffer", 1],
        ["light-shadows", 4],
      ])
    );
    timer.record(new Map([["gbuffer", 1]]));
    timer.record(new Map([["gbuffer", 1]]));
    timer.record(new Map([["gbuffer", 1]]));

    expect(timer.describe(["gbuffer", "light-shadows"])).toEqual([
      { gpuTime: 1, name: "gbuffer" },
      { gpuTime: 1, name: "light-shadows" },
    ]);
  });

  it("reports zero for a pass never timed", () => {
    expect(new RendererPassTimer().describe(["clear"])).toEqual([{ gpuTime: 0, name: "clear" }]);
  });

  it("averages over the last thirty frames only", () => {
    const timer: RendererPassTimer = new RendererPassTimer();

    for (let frame = 0; frame < 30; frame++) {
      timer.record(new Map([["clear", 100]]));
    }

    for (let frame = 0; frame < 30; frame++) {
      timer.record(new Map([["clear", 1]]));
    }

    expect(timer.describe(["clear"])).toEqual([{ gpuTime: 1, name: "clear" }]);
  });

  it("forgets everything on reset", () => {
    const timer: RendererPassTimer = new RendererPassTimer();

    timer.record(new Map([["clear", 5]]));
    timer.reset();

    expect(timer.describe(["clear"])).toEqual([{ gpuTime: 0, name: "clear" }]);
  });
});
