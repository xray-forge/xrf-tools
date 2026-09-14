import { EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";
import { TEquipmentCell } from "@/core/sprite-equipment/lib/equipment";
import { IEquipmentGrid, isOccupantOutsideSheet, toEquipmentGrid } from "@/core/sprite-equipment/lib/grid";

/** Nothing occupies most cells, and every caller reads the same array back for them. */
const NOTHING: ReadonlyArray<EquipmentSlotOccupant> = [];

/** One sheet as the editor reads it: the lattice, and what every cell of it holds. */
export interface IEquipmentLayout {
  /** The lattice the sheet is read through. */
  grid: IEquipmentGrid;
  /** Every section occupying a slot, in declaration order. */
  occupants: ReadonlyArray<EquipmentSlotOccupant>;
  /** The rectangles that leave the image, kept and reported rather than discarded. */
  outside: ReadonlyArray<EquipmentSlotOccupant>;
  /**  Rectangles covering a cell, in declaration order. */
  at(cell: TEquipmentCell): ReadonlyArray<EquipmentSlotOccupant>;
}

/**
 * Reads one sheet and the configuration's claims on it.
 *
 * @param width - Sheet width in pixels.
 * @param height - Sheet height in pixels.
 * @param size - Cell side in pixels.
 * @param occupants - Rectangles the configuration declares, measured in cells.
 * @returns The lattice and its occupants.
 */
export function toEquipmentLayout(
  width: number,
  height: number,
  size: number,
  occupants: ReadonlyArray<EquipmentSlotOccupant>
): IEquipmentLayout {
  const grid: IEquipmentGrid = toEquipmentGrid(width, height, size, occupants);

  // Sparse: a dense row-by-column array of Anomaly's lattice is over thirteen thousand entries to describe two
  // thousand rectangles, and every empty one of them is allocated and walked.
  const cells: Map<number, Array<EquipmentSlotOccupant>> = new Map();
  const outside: Array<EquipmentSlotOccupant> = [];

  for (const occupant of occupants) {
    if (isOccupantOutsideSheet(grid, occupant)) {
      outside.push(occupant);
    }

    for (let row = occupant.y; row < occupant.y + occupant.h; row++) {
      for (let column = occupant.x; column < occupant.x + occupant.w; column++) {
        const key: number = row * grid.columns + column;
        const held: Array<EquipmentSlotOccupant> | undefined = cells.get(key);

        if (held) {
          held.push(occupant);
        } else {
          cells.set(key, [occupant]);
        }
      }
    }
  }

  return {
    at([row, column]: TEquipmentCell): ReadonlyArray<EquipmentSlotOccupant> {
      // Unique per cell because the lattice spans every occupant it was built from, which is what building both
      // here guarantees.
      return cells.get(row * grid.columns + column) ?? NOTHING;
    },
    occupants,
    grid,
    outside,
  };
}
