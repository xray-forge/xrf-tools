import { Nullable } from "@xrf/types";

import { EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";
import { IEquipmentLayout, TEquipmentCell } from "@/core/sprite-equipment/lib";

/** One occupant as the list shows it, with the cell revealing it lands on. */
export interface IEquipmentOccupantRow {
  occupant: EquipmentSlotOccupant;
  /** Top left cell of the rectangle, which is where revealing this row goes. */
  cell: TEquipmentCell;
}

/**
 * Every occupant of a sheet, in declaration order, filtered by what someone typed.
 *
 * @param layout - The lattice and its occupants, or null while nothing is open.
 * @param filter - What to match, or empty for everything.
 * @returns The rows to list.
 */
export function toEquipmentOccupantRows(
  layout: Nullable<IEquipmentLayout>,
  filter: string
): Array<IEquipmentOccupantRow> {
  if (!layout) {
    return [];
  }

  const needle: string = filter.trim().toLowerCase();

  return layout.occupants
    .filter(
      (occupant: EquipmentSlotOccupant) =>
        !needle ||
        occupant.section.toLowerCase().includes(needle) ||
        (occupant.origin?.toLowerCase().includes(needle) ?? false)
    )
    .map((occupant: EquipmentSlotOccupant) => ({ occupant, cell: [occupant.y, occupant.x] as TEquipmentCell }));
}
