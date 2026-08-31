import { IEquipmentSectionDescriptor } from "@/core/sprite-equipment/equipment";

/**
 * Maps sprite rectangles to the fixed-size cells they cover.
 *
 * Coordinates and dimensions in descriptors are cell units. Overlapping descriptors are retained in insertion order
 * for each cell, allowing the editor to explain every sprite that covers a selected location.
 */
export class GridMapper {
  /** Number of rows that fit in the sprite height. */
  public rows: number;
  /** Number of columns that fit in the sprite width. */
  public columns: number;
  /** Descriptors indexed as `[row][column]`; empty cells are `null`. */
  public grid: Array<Array<Array<IEquipmentSectionDescriptor>>>;
  /** Cell side length in sprite pixels. */
  public gridSize: number;

  /**
   * Builds the cell map, ignoring descriptors that extend beyond the grid.
   *
   * @param width - Sprite width in pixels.
   * @param height - Sprite height in pixels.
   * @param size - Square cell size in pixels.
   * @param descriptors - Sprite rectangles measured in cells.
   */
  public constructor(width: number, height: number, size: number, descriptors: Array<IEquipmentSectionDescriptor>) {
    const rows: number = Math.floor(height / size);
    const columns: number = Math.floor(width / size);
    const grid: Array<Array<Array<IEquipmentSectionDescriptor>>> = new Array(rows);

    for (let it = 0; it < rows; it++) {
      grid[it] = new Array(columns).fill(null);
    }

    descriptors.forEach((it) => {
      if (it.x + it.w <= columns && it.y + it.h <= rows) {
        for (let i = it.x; i < it.x + it.w; i++) {
          for (let j = it.y; j < it.y + it.h; j++) {
            if (grid[j][i]) {
              grid[j][i].push(it);
            } else {
              grid[j][i] = [it];
            }
          }
        }
      }
    });

    this.grid = grid;
    this.rows = rows;
    this.columns = columns;
    this.gridSize = size;
  }
}
