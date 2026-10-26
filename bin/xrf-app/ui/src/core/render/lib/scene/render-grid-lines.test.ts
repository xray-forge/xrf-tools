import { describe, expect, it } from "@jest/globals";
import { RenderOverlaySegment } from "@xrf/ipc/types/xrf-renderer";
import { toRawColor } from "@xrf/math";

import { toRenderAxesLines, toRenderBoxLines, toRenderGridLines } from "@/core/render/lib/scene/render-grid-lines";

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

describe("toRenderBoxLines", () => {
  it("draws the box's twelve edges, each along one axis and as long as the box is that way", () => {
    const segments: Array<RenderOverlaySegment> = toRenderBoxLines(
      { x: -1, y: 0, z: 2 },
      { x: 3, y: 5, z: 8 },
      0xff8000
    );
    const sizes: [number, number, number] = [4, 5, 6];

    expect(segments).toHaveLength(12);

    for (const { colors, ends } of segments) {
      const deltas: Array<number> = ends[1].map((end, axis: number) => (end ?? 0) - (ends[0][axis] ?? 0));
      const axis: number = deltas.findIndex((delta: number) => delta !== 0);

      expect(deltas.filter((delta: number) => delta !== 0)).toHaveLength(1);
      expect(deltas[axis]).toBe(sizes[axis]);
      expect(colors).toEqual([toRawColor(0xff8000), toRawColor(0xff8000)]);
    }
  });
});

describe("toRenderAxesLines", () => {
  it("runs each axis from the origin, in its colour fading towards its tip", () => {
    const segments: Array<RenderOverlaySegment> = toRenderAxesLines(2);

    expect(segments).toHaveLength(3);
    expect(segments[1]).toEqual({
      colors: [
        [0, 1, 0],
        [0.797, 1, 0],
      ],
      ends: [
        [0, 0, 0],
        [0, 2, 0],
      ],
    });
  });
});
