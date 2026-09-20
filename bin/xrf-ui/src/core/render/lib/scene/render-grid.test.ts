import { describe, expect, it } from "@jest/globals";
import { GridHelper } from "three";

import { RenderGrid } from "./render-grid";

function gridOf(grid: RenderGrid): GridHelper {
  return grid.object.children[0] as GridHelper;
}

describe("RenderGrid", () => {
  it("draws cells of a round size across the extent it is given", () => {
    const grid: RenderGrid = new RenderGrid({ cells: 40, color: 0x111111, originColor: 0x222222 });

    grid.setExtent(700);

    expect(grid.step).toBe(50);
    expect(grid.object.visible).toBe(true);
  });

  // Scaling keeps the line count, which is what made one implementation unable to serve both viewers: a level would
  // have been crossed by the twenty-five lines a rifle is, in cells of no particular size.
  it("rebuilds rather than scales, and keeps the group it was added to", () => {
    const grid: RenderGrid = new RenderGrid({ cells: 40, color: 0x111111, originColor: 0x222222 });
    const before: GridHelper = gridOf(grid);

    grid.setExtent(700);

    expect(grid.object.children).toHaveLength(1);
    expect(gridOf(grid)).not.toBe(before);
    expect(gridOf(grid).scale.x).toBe(1);
  });

  it("keeps the grid it has when the extent asks for the same step", () => {
    const grid: RenderGrid = new RenderGrid({ cells: 40, color: 0x111111, originColor: 0x222222 });

    grid.setExtent(700);

    const rebuilt: GridHelper = gridOf(grid);

    grid.setExtent(710);

    expect(gridOf(grid)).toBe(rebuilt);
  });

  it("holds its visibility across a resize", () => {
    const grid: RenderGrid = new RenderGrid({ cells: 40, color: 0x111111, originColor: 0x222222 });

    grid.setVisible(false);
    grid.setExtent(700);

    expect(grid.object.visible).toBe(false);
  });

  // The grid lies on the ground it measures, so it must not fight the surface drawn on that same plane.
  it("writes no depth", () => {
    const grid: RenderGrid = new RenderGrid({ cells: 40, color: 0x111111, originColor: 0x222222 });

    expect(gridOf(grid).material.depthWrite).toBe(false);
  });
});
