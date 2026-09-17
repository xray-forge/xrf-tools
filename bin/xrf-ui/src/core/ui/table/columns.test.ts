import { describe, expect, it } from "@jest/globals";
import { GridColDef } from "@mui/x-data-grid";

import {
  decimalColumn,
  flagsColumn,
  identifierColumn,
  textColumn,
  tupleColumn,
  vectorColumn,
} from "@/core/ui/table/columns";

/**
 * Reads a value through a column's value getter.
 *
 * @param column - Column whose value getter to invoke.
 * @param value - Cell value passed as the getter's first argument.
 * @returns Value returned by the getter, if present.
 */
function readCell(column: GridColDef, value: unknown): unknown {
  return column.valueGetter?.(value as never, {} as never, column, {} as never);
}

describe("table columns", () => {
  it("leaves a plain column without a getter", () => {
    expect(textColumn("version", "Version")).toEqual({ field: "version", headerName: "Version" });
    expect(textColumn("version", "Version", 120).width).toBe(120);
  });

  it("marks identifiers monospace, which is what makes them comparable by eye", () => {
    expect(identifierColumn("guid", "Guid").cellClassName).toBe("monospace");
  });

  it("renders a vector as its components rather than as JSON", () => {
    const column: GridColDef = vectorColumn("position", "Position");

    // Was `{"x":12.5,"y":1.25,"z":-30}` in every cell, spending the width on punctuation.
    expect(readCell(column, { x: 12.5, y: 1.25, z: -30 })).toBe("12.50, 1.25, -30.00");
  });

  it("leaves a missing vector empty rather than printing null", () => {
    expect(readCell(vectorColumn("position", "Position"), undefined)).toBeNull();
    expect(readCell(vectorColumn("position", "Position"), 4)).toBeNull();
  });

  it("renders flags as hex, since decimal bit fields cannot be read as flags", () => {
    expect(readCell(flagsColumn("scriptFlags", "Script flags"), 255)).toBe("0xFF");
    expect(readCell(flagsColumn("scriptFlags", "Script flags"), 0)).toBe("0x0");
    expect(readCell(flagsColumn("scriptFlags", "Script flags"), null)).toBeNull();
  });

  it("renders a tuple as a compact list", () => {
    expect(readCell(tupleColumn("vertexType", "Vertex type"), [1, 2, 3, 4])).toBe("1, 2, 3, 4");
    expect(readCell(tupleColumn("vertexType", "Vertex type"), null)).toBeNull();
  });

  it("renders a float at fixed precision", () => {
    const column: GridColDef = decimalColumn("distance", "Distance");

    expect(readCell(column, 1.23456)).toBe(1.23456);
    expect(readCell(column, null)).toBeNull();
    expect(column.valueFormatter?.(1.23456 as never, {} as never, column, {} as never)).toBe("1.23");
    expect(column.valueFormatter?.(null as never, {} as never, column, {} as never)).toBeNull();
  });
});
