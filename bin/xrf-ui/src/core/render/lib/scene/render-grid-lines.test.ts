import { describe, expect, it } from "@jest/globals";

import { toRenderGridLines } from "@/core/render/lib/scene/render-grid-lines";

/** Floats one grid line takes: two segments of two vertices each, three floats a vertex. */
const FLOATS_PER_LINE: number = 12;

describe("toRenderGridLines", () => {
  it("crosses the extent at a round step", () => {
    const { positions } = toRenderGridLines(100, { cells: 10, color: 0, originColor: 0 });

    // A step of twenty over two hundred metres: eleven lines, the first on the edge.
    expect(positions).toHaveLength(11 * FLOATS_PER_LINE);
    expect(positions[2]).toBe(-100);
    expect(positions[FLOATS_PER_LINE + 2]).toBe(-80);
  });

  // Half the cell and twice the lines, so the finer measure still reaches as far.
  it("splits each cell as asked, reaching the same edge", () => {
    const { positions } = toRenderGridLines(100, { cells: 10, color: 0, originColor: 0, subdivision: 2 });

    expect(positions).toHaveLength(21 * FLOATS_PER_LINE);
    expect(positions[2]).toBe(-100);
    expect(positions[FLOATS_PER_LINE + 2]).toBe(-90);
    expect(positions.at(-1)).toBe(100);
  });

  it("lays the grid around the centre it is given", () => {
    const { positions } = toRenderGridLines(100, {
      cells: 10,
      center: { x: 5, y: 2, z: -5 },
      color: 0,
      originColor: 0,
    });

    expect(Array.from(positions.slice(0, 3))).toEqual([-95, 2, -105]);
  });
});
