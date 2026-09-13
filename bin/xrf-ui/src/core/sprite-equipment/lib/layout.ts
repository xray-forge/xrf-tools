import { IEquipmentSectionDescriptor, TEquipmentCell } from "@/core/sprite-equipment/lib/equipment";
import { IEquipmentGrid, isDescriptorOutsideSheet, toEquipmentGrid } from "@/core/sprite-equipment/lib/grid";

/** Nothing occupies most cells, and every caller reads the same array back for them. */
const NOTHING: ReadonlyArray<IEquipmentSectionDescriptor> = [];

/** One sheet as the editor reads it: the lattice, and what every cell of it holds. */
export interface IEquipmentLayout {
  /** The lattice the sheet is read through. */
  grid: IEquipmentGrid;
  /** Every rectangle the configuration declares, in declaration order. */
  descriptors: ReadonlyArray<IEquipmentSectionDescriptor>;
  /** The rectangles that leave the image, kept and reported rather than discarded. */
  outside: ReadonlyArray<IEquipmentSectionDescriptor>;
  /**  Rectangles covering a cell, in declaration order. */
  at(cell: TEquipmentCell): ReadonlyArray<IEquipmentSectionDescriptor>;
}

/**
 * Reads one sheet and the configuration's claims on it.
 *
 * @param width - Sheet width in pixels.
 * @param height - Sheet height in pixels.
 * @param size - Cell side in pixels.
 * @param descriptors - Rectangles the configuration declares, measured in cells.
 * @returns The lattice and its occupants.
 */
export function toEquipmentLayout(
  width: number,
  height: number,
  size: number,
  descriptors: ReadonlyArray<IEquipmentSectionDescriptor>
): IEquipmentLayout {
  const grid: IEquipmentGrid = toEquipmentGrid(width, height, size, descriptors);

  // Sparse: a dense row-by-column array of Anomaly's lattice is over thirteen thousand entries to describe two
  // thousand rectangles, and every empty one of them is allocated and walked.
  const cells: Map<number, Array<IEquipmentSectionDescriptor>> = new Map();
  const outside: Array<IEquipmentSectionDescriptor> = [];

  for (const descriptor of descriptors) {
    if (isDescriptorOutsideSheet(grid, descriptor)) {
      outside.push(descriptor);
    }

    for (let row = descriptor.y; row < descriptor.y + descriptor.h; row++) {
      for (let column = descriptor.x; column < descriptor.x + descriptor.w; column++) {
        const key: number = row * grid.columns + column;
        const held: Array<IEquipmentSectionDescriptor> | undefined = cells.get(key);

        if (held) {
          held.push(descriptor);
        } else {
          cells.set(key, [descriptor]);
        }
      }
    }
  }

  return {
    at([row, column]: TEquipmentCell): ReadonlyArray<IEquipmentSectionDescriptor> {
      // Unique per cell because the lattice spans every descriptor it was built from, which is what building both
      // here guarantees.
      return cells.get(row * grid.columns + column) ?? NOTHING;
    },
    descriptors,
    grid,
    outside,
  };
}
