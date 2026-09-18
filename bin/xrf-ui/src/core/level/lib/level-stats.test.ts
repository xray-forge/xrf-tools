import { describe, expect, it } from "@jest/globals";

import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { createSectorViews } from "@/core/level/lib/level-sector-views";
import { EMPTY_LEVEL_STATS, ILevelStats, LevelFrameTimer, measureLevelStats } from "@/core/level/lib/level-stats";
import { mockSectorDescription, mockSectorSection } from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

/** One resident sector drawing the given surfaces, without a renderer to upload it. */
function loadedSector(sector: number, sections: number): ILoadedSector {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description = mockSectorDescription(buffer, {
    sector,
    sections: Array.from({ length: sections }, (_, index) =>
      mockSectorSection({ draw: { count: 3, start: index * 3 }, shaderId: index })
    ),
  });

  return {
    geometry: { dispose: () => undefined } as never,
    sector,
    views: createSectorViews({ ...description, bufferLength: buffer.byteLength }, buffer.toArrayBuffer()),
  };
}

describe("level stats", () => {
  it("measures nothing when nothing is resident", () => {
    expect(measureLevelStats(new Map(), 0)).toEqual(EMPTY_LEVEL_STATS);
  });

  // A draw call per surface per sector is the cost the residency budget is really buying.
  it("counts a draw for each surface of each resident sector", () => {
    const sectors: Map<number, ILoadedSector> = new Map([
      [0, loadedSector(0, 2)],
      [1, loadedSector(1, 3)],
    ]);

    const stats: ILevelStats = measureLevelStats(sectors, 16);

    expect(stats.sectors).toBe(2);
    expect(stats.draws).toBe(5);
    expect(stats.triangles).toBe(5);
    expect(stats.bytes).toBeGreaterThan(0);
  });

  it("derives a frame rate from the mean frame time", () => {
    expect(measureLevelStats(new Map(), 20).framesPerSecond).toBeCloseTo(50);
  });

  it("reports no frame rate before a frame has been timed", () => {
    expect(measureLevelStats(new Map(), 0).framesPerSecond).toBe(0);
  });
});

describe("LevelFrameTimer", () => {
  // The first frame has nothing to measure against, so reporting one would be inventing it.
  it("reports nothing until it has seen two frames", () => {
    const timer: LevelFrameTimer = new LevelFrameTimer();

    timer.sample(1000);

    expect(timer.frameTime).toBe(0);
  });

  it("averages the frames it has seen", () => {
    const timer: LevelFrameTimer = new LevelFrameTimer();

    timer.sample(0);
    timer.sample(10);
    timer.sample(30);

    expect(timer.frameTime).toBe(15);
  });

  it("forgets the window when the viewport does", () => {
    const timer: LevelFrameTimer = new LevelFrameTimer();

    timer.sample(0);
    timer.sample(10);
    timer.reset();

    expect(timer.frameTime).toBe(0);
  });
});
