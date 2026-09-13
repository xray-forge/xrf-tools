import { IEquipmentSectionDescriptor, TEquipmentCell } from "@/core/sprite-equipment/lib/equipment";
import { Nullable } from "@/lib/types/general";

/** A rectangle in sheet pixels. */
export interface IEquipmentRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/** The cell lattice a sheet is read through. */
export interface IEquipmentGrid {
  /** Cell side in sheet pixels. */
  size: number;
  /** Columns the lattice spans, covering the sheet and everything the configuration claims. */
  columns: number;
  /** Rows the lattice spans. */
  rows: number;
  /** Columns the image itself covers. A cell at or past this is outside the sheet. */
  sheetColumns: number;
  /** Rows the image itself covers. */
  sheetRows: number;
}

/**
 * Builds the lattice for one sheet and the rectangles claimed on it.
 *
 * @param width - Sheet width in pixels.
 * @param height - Sheet height in pixels.
 * @param size - Cell side in pixels.
 * @param descriptors - Rectangles the configuration declares, measured in cells.
 * @returns The lattice, spanning the image and every claimed rectangle.
 */
export function toEquipmentGrid(
  width: number,
  height: number,
  size: number,
  descriptors: ReadonlyArray<IEquipmentSectionDescriptor>
): IEquipmentGrid {
  // A partly covered cell is still a cell someone can point at, so the image rounds up rather than down. Flooring is
  // what dropped Anomaly's last row, where eighty-two cells of fifty pixels overrun a 4096 pixel sheet by four.
  const sheetColumns: number = size > 0 ? Math.ceil(width / size) : 0;
  const sheetRows: number = size > 0 ? Math.ceil(height / size) : 0;

  let columns: number = sheetColumns;
  let rows: number = sheetRows;

  for (const descriptor of descriptors) {
    columns = Math.max(columns, descriptor.x + descriptor.w);
    rows = Math.max(rows, descriptor.y + descriptor.h);
  }

  return { columns, rows, sheetColumns, sheetRows, size };
}

/**
 * Where a cell sits on the sheet.
 *
 * @param grid - Lattice the cell belongs to.
 * @param cell - Cell as `[row, column]`.
 * @returns Its rectangle in sheet pixels.
 */
export function toCellRect(grid: IEquipmentGrid, cell: TEquipmentCell): IEquipmentRect {
  const [row, column] = cell;

  return { x: column * grid.size, y: row * grid.size, width: grid.size, height: grid.size };
}

/**
 * Where a declared rectangle sits on the sheet.
 *
 * @param grid - Lattice the rectangle is measured against.
 * @param descriptor - Rectangle in cells.
 * @returns Its rectangle in sheet pixels.
 */
export function toDescriptorRect(grid: IEquipmentGrid, descriptor: IEquipmentSectionDescriptor): IEquipmentRect {
  return {
    x: descriptor.x * grid.size,
    y: descriptor.y * grid.size,
    width: descriptor.w * grid.size,
    height: descriptor.h * grid.size,
  };
}

/**
 * The parts of the lattice no image covers.
 *
 * @param grid - Lattice to measure.
 * @returns Rectangles in sheet pixels, in no particular order.
 */
export function toOutsideSheetRects(grid: IEquipmentGrid): Array<IEquipmentRect> {
  const sheetWidth: number = grid.sheetColumns * grid.size;
  const sheetHeight: number = grid.sheetRows * grid.size;
  const width: number = grid.columns * grid.size;
  const height: number = grid.rows * grid.size;

  const rects: Array<IEquipmentRect> = [];

  if (width > sheetWidth) {
    rects.push({ x: sheetWidth, y: 0, width: width - sheetWidth, height });
  }

  if (height > sheetHeight) {
    rects.push({ x: 0, y: sheetHeight, width: sheetWidth, height: height - sheetHeight });
  }

  return rects;
}

/**
 * Which cell a point on the sheet falls in.
 *
 * @param grid - Lattice to read the point through.
 * @param x - Horizontal position in sheet pixels.
 * @param y - Vertical position in sheet pixels.
 * @returns The cell, or null when the point falls outside the lattice entirely.
 */
export function toCellAt(grid: IEquipmentGrid, x: number, y: number): Nullable<TEquipmentCell> {
  if (grid.size <= 0 || x < 0 || y < 0) {
    return null;
  }

  const column: number = Math.floor(x / grid.size);
  const row: number = Math.floor(y / grid.size);

  return column < grid.columns && row < grid.rows ? [row, column] : null;
}

/**
 * Whether a cell lies beyond the image the lattice was built over.
 *
 * @param grid - Lattice the cell belongs to.
 * @param cell - Cell as `[row, column]`.
 * @returns Whether nothing on the sheet can be drawn there.
 */
export function isCellOutsideSheet(grid: IEquipmentGrid, cell: TEquipmentCell): boolean {
  const [row, column] = cell;

  return column >= grid.sheetColumns || row >= grid.sheetRows;
}

/**
 * Whether a declared rectangle leaves the image.
 *
 * @param grid - Lattice the rectangle is measured against.
 * @param descriptor - Rectangle in cells.
 * @returns Whether any part of it falls outside the sheet.
 */
export function isDescriptorOutsideSheet(grid: IEquipmentGrid, descriptor: IEquipmentSectionDescriptor): boolean {
  return descriptor.x + descriptor.w > grid.sheetColumns || descriptor.y + descriptor.h > grid.sheetRows;
}

/**
 * Whether two cells name the same place.
 *
 * Cells are tuples built fresh on every lookup, so a consumer holding one in state compares by value or re-renders on
 * every pointer move across one cell.
 *
 * @param first - A cell, or nothing.
 * @param second - A cell, or nothing.
 * @returns Whether both name the same cell, including both being nothing.
 */
export function isSameCell(first: Nullable<TEquipmentCell>, second: Nullable<TEquipmentCell>): boolean {
  return first === second || (first !== null && second !== null && first[0] === second[0] && first[1] === second[1]);
}
