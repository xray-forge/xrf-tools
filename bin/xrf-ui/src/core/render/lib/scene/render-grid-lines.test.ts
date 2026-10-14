import { describe, expect, it } from "@jest/globals";

import { RenderOverlaySegment } from "@/core/ipc/types/xrf-renderer";
import { toRenderGridLines } from "@/core/render/lib/scene/render-grid-lines";

/** Segments one grid line takes: one along each axis. */
const SEGMENTS_PER_LINE: number = 2;

describe("toRenderGridLines", () => {
  it("crosses the extent at a round step", () => {
    const segments: Array<RenderOverlaySegment> = toRenderGridLines(100, { cells: 10, color: 0, originColor: 0 });

    // A step of twenty over two hundred metres: eleven lines, the first on the edge.
    expect(segments).toHaveLength(11 * SEGMENTS_PER_LINE);
    expect(segments[0].ends[0][2]).toBe(-100);
    expect(segments[SEGMENTS_PER_LINE].ends[0][2]).toBe(-80);
  });

  // Half the cell and twice the lines, so the finer measure still reaches as far.
  it("splits each cell as asked, reaching the same edge", () => {
    const segments: Array<RenderOverlaySegment> = toRenderGridLines(100, {
      cells: 10,
      color: 0,
      originColor: 0,
      subdivision: 2,
    });

    expect(segments).toHaveLength(21 * SEGMENTS_PER_LINE);
    expect(segments[0].ends[0][2]).toBe(-100);
    expect(segments[SEGMENTS_PER_LINE].ends[0][2]).toBe(-90);
    expect(segments.at(-1)?.ends[1][2]).toBe(100);
  });

  it("lays the grid around the centre it is given", () => {
    const segments: Array<RenderOverlaySegment> = toRenderGridLines(100, {
      cells: 10,
      center: { x: 5, y: 2, z: -5 },
      color: 0,
      originColor: 0,
    });

    expect(segments[0].ends[0]).toEqual([-95, 2, -105]);
  });
});
