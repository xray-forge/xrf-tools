import { describe, expect, it } from "@jest/globals";

import { IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";

import { EMPTY_LEVEL_STATS, ILevelStats, measureLevelStats } from "./level-stats";

/** What a renderer counted for the frame just drawn. */
function frameCost(overrides: Partial<IRenderFrameCost> = {}): IRenderFrameCost {
  return {
    drawTime: 0,
    draws: 0,
    frameTime: 0,
    framesPerSecond: 0,
    triangles: 0,
    worstDrawTime: 0,
    worstFrameTime: 0,
    ...overrides,
  };
}

describe("level stats", () => {
  it("measures nothing when nothing is resident and nothing has been drawn", () => {
    expect(measureLevelStats({ bytes: 0, sectors: 0 }, frameCost())).toEqual(EMPTY_LEVEL_STATS);
  });

  // What is held is the sector set's question - it is the only thing that can see a sector - and this only
  // carries the answer into the report.
  it("carries what is held into the report", () => {
    const stats: ILevelStats = measureLevelStats({ bytes: 4096, sectors: 2 }, frameCost());

    expect(stats.sectors).toBe(2);
    expect(stats.bytes).toBe(4096);
  });

  // What a frame cost is the renderer's question, and the two stopped agreeing the moment anything was culled:
  // counting the held draws and calling that the frame's cost reports the number culling exists to reduce.
  it("takes the draws and triangles from what the renderer counted, not from what is held", () => {
    const stats: ILevelStats = measureLevelStats({ bytes: 4096, sectors: 1 }, frameCost({ draws: 7, triangles: 120 }));

    expect(stats.draws).toBe(7);
    expect(stats.triangles).toBe(120);
  });

  it("reports the frame timing the viewport measured", () => {
    const stats: ILevelStats = measureLevelStats(
      { bytes: 0, sectors: 0 },
      frameCost({ frameTime: 20, framesPerSecond: 50 })
    );

    expect(stats.frameTime).toBe(20);
    expect(stats.framesPerSecond).toBe(50);
  });
});
