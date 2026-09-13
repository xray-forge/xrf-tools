import { describe, expect, it } from "@jest/globals";

import {
  IEquipmentGrid,
  isCellOutsideSheet,
  isDescriptorOutsideSheet,
  isSameCell,
  toCellAt,
  toCellRect,
  toDescriptorRect,
  toEquipmentGrid,
  toOutsideSheetRects,
} from "@/core/sprite-equipment/lib";
import { mockEquipmentDescriptor } from "@/fixtures/mocks/sprite.mocks";

describe("toEquipmentGrid", () => {
  it("spans the sheet when nothing is claimed beyond it", () => {
    expect(toEquipmentGrid(1024, 2048, 50, [mockEquipmentDescriptor("wpn_ak74")])).toEqual({
      columns: 21,
      rows: 41,
      sheetColumns: 21,
      sheetRows: 41,
      size: 50,
    });
  });

  it("counts a partly covered cell as a cell", () => {
    // Anomaly's sheet is 4096 tall and its last icons sit in row 82, which ends four pixels past it. Flooring here is
    // what dropped that row entirely.
    const grid: IEquipmentGrid = toEquipmentGrid(8192, 4096, 50, []);

    expect(grid.sheetColumns).toBe(164);
    expect(grid.sheetRows).toBe(82);
  });

  it("extends past the sheet to cover everything the configuration claims", () => {
    // CoC declares rectangles reaching 51 cells against a sheet only 21 wide. The lattice has to hold them, because a
    // rectangle nobody draws is a rectangle nobody can be told about.
    const grid: IEquipmentGrid = toEquipmentGrid(1024, 2048, 50, [
      mockEquipmentDescriptor("af_medusa", { x: 48, y: 48, w: 3, h: 3 }),
    ]);

    expect(grid).toEqual({ columns: 51, rows: 51, sheetColumns: 21, sheetRows: 41, size: 50 });
  });

  it("survives a sheet with no size yet", () => {
    expect(toEquipmentGrid(0, 0, 50, [])).toEqual({
      columns: 0,
      rows: 0,
      sheetColumns: 0,
      sheetRows: 0,
      size: 50,
    });
  });
});

describe("toCellAt", () => {
  const grid: IEquipmentGrid = toEquipmentGrid(1000, 500, 50, []);

  it("answers the cell a point falls in", () => {
    expect(toCellAt(grid, 0, 0)).toEqual([0, 0]);
    expect(toCellAt(grid, 49, 49)).toEqual([0, 0]);
    expect(toCellAt(grid, 50, 50)).toEqual([1, 1]);
    expect(toCellAt(grid, 275, 125)).toEqual([2, 5]);
  });

  it("answers nothing outside the lattice", () => {
    expect(toCellAt(grid, -1, 10)).toBeNull();
    expect(toCellAt(grid, 10, -1)).toBeNull();
    expect(toCellAt(grid, 1000, 10)).toBeNull();
    expect(toCellAt(grid, 10, 500)).toBeNull();
  });

  it("answers nothing rather than dividing by a cell size of zero", () => {
    expect(toCellAt({ columns: 0, rows: 0, sheetColumns: 0, sheetRows: 0, size: 0 }, 10, 10)).toBeNull();
  });
});

describe("toCellRect and toDescriptorRect", () => {
  const grid: IEquipmentGrid = toEquipmentGrid(1000, 500, 50, []);

  it("places a cell on the sheet", () => {
    expect(toCellRect(grid, [4, 3])).toEqual({ x: 150, y: 200, width: 50, height: 50 });
  });

  it("places a declared rectangle on the sheet", () => {
    expect(toDescriptorRect(grid, mockEquipmentDescriptor("wpn_ak74", { x: 25, y: 4, w: 5, h: 2 }))).toEqual({
      x: 1250,
      y: 200,
      width: 250,
      height: 100,
    });
  });
});

describe("toOutsideSheetRects", () => {
  it("answers nothing when the sheet covers the lattice", () => {
    expect(toOutsideSheetRects(toEquipmentGrid(1000, 500, 50, []))).toEqual([]);
  });

  it("answers one strip when a single axis overruns", () => {
    const grid: IEquipmentGrid = toEquipmentGrid(1000, 500, 50, [
      mockEquipmentDescriptor("over_the_right_edge", { x: 24, y: 0 }),
    ]);

    expect(toOutsideSheetRects(grid)).toEqual([{ x: 1000, y: 0, width: 250, height: 500 }]);
  });

  it("answers disjoint strips when both axes overrun", () => {
    const grid: IEquipmentGrid = toEquipmentGrid(1000, 500, 50, [mockEquipmentDescriptor("far", { x: 24, y: 12 })]);

    // Disjoint on purpose: the overlay paints these over the picture, so a bounding box would tint the very image they
    // are meant to lie outside of.
    expect(toOutsideSheetRects(grid)).toEqual([
      { x: 1000, y: 0, width: 250, height: 650 },
      { x: 0, y: 500, width: 1000, height: 150 },
    ]);
  });
});

describe("outside the sheet", () => {
  // Twenty-one columns and forty-one rows of image, with a rectangle claiming three cells past the right edge.
  const grid: IEquipmentGrid = toEquipmentGrid(1024, 2048, 50, [
    mockEquipmentDescriptor("af_medusa", { x: 48, y: 48, w: 3, h: 3 }),
  ]);

  it("knows which cells the image cannot cover", () => {
    expect(isCellOutsideSheet(grid, [0, 0])).toBe(false);
    expect(isCellOutsideSheet(grid, [40, 20])).toBe(false);
    expect(isCellOutsideSheet(grid, [0, 21])).toBe(true);
    expect(isCellOutsideSheet(grid, [41, 0])).toBe(true);
  });

  it("knows which rectangles leave the image", () => {
    expect(isDescriptorOutsideSheet(grid, mockEquipmentDescriptor("inside"))).toBe(false);
    expect(isDescriptorOutsideSheet(grid, mockEquipmentDescriptor("last", { x: 20, y: 40 }))).toBe(false);
    // Starting inside and ending outside still leaves it, which is the case a bounds check on the origin misses.
    expect(isDescriptorOutsideSheet(grid, mockEquipmentDescriptor("straddling", { x: 20, y: 40, w: 2 }))).toBe(true);
  });
});

describe("isSameCell", () => {
  it("compares cells by value, because every lookup builds a new tuple", () => {
    expect(isSameCell([1, 2], [1, 2])).toBe(true);
    expect(isSameCell([1, 2], [2, 1])).toBe(false);
    expect(isSameCell(null, null)).toBe(true);
    expect(isSameCell([1, 2], null)).toBe(false);
    expect(isSameCell(null, [1, 2])).toBe(false);
  });
});
