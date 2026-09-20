import { describe, expect, it } from "@jest/globals";

import { ILoadedSector } from "@/core/level/lib/level-sector-set";
import { createSectorViews } from "@/core/level/lib/level-sector-views";
import { EMPTY_LEVEL_STATS, ILevelStats, measureLevelStats } from "@/core/level/lib/level-stats";
import {
  mockSectorDescription,
  mockSectorInstanceGroup,
  mockSectorSection,
  mockSectorSurface,
} from "@/fixtures/mocks/level.mocks";
import { MockVisualBuffer } from "@/fixtures/mocks/visual.mocks";

/** One resident sector drawing the given surfaces, without a renderer to upload it. */
function loadedSector(sector: number, sections: number): ILoadedSector {
  const buffer: MockVisualBuffer = new MockVisualBuffer();
  const description = mockSectorDescription(buffer, {
    sector,
    sections: Array.from({ length: sections }, (_, index) =>
      mockSectorSection({ draw: { count: 3, start: index * 3 }, surface: mockSectorSurface({ shaderId: index }) })
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

describe("instanced geometry", () => {
  /** One resident sector standing one triangle in `places` places, beside `sections` baked surfaces. */
  function standingSector(sections: number, places: number): ILoadedSector {
    const buffer: MockVisualBuffer = new MockVisualBuffer();
    const description = mockSectorDescription(buffer, {
      instances: [
        mockSectorInstanceGroup(
          buffer,
          Array.from({ length: places }, (_, at: number) => at)
        ),
      ],
      sections: Array.from({ length: sections }, (_, index) =>
        mockSectorSection({ draw: { count: 3, start: index * 3 }, surface: mockSectorSurface({ shaderId: index }) })
      ),
    });

    return {
      geometry: { dispose: () => undefined } as never,
      sector: 0,
      views: createSectorViews({ ...description, bufferLength: buffer.byteLength }, buffer.toArrayBuffer()),
    };
  }

  // The panel counted sections alone, so marsh's 358 instanced meshes and the 10,973 places they stand were invisible
  // to the one measurement the streaming work exists to make.
  it("counts an instanced mesh as a draw, and its triangles once for every place it stands", () => {
    const stats: ILevelStats = measureLevelStats(new Map([[0, standingSector(2, 7)]]), 16);

    expect(stats.draws).toBe(3);
    expect(stats.triangles).toBe(2 + 7);
  });
});
