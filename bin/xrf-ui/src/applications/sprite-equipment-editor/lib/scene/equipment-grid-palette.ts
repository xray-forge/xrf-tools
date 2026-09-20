import { alpha, Theme } from "@mui/material";

import { EEquipmentSlotClaim, EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";

/**
 * What each part of the lattice is drawn in.
 *
 * Alpha throughout, because all of it is drawn over the picture being inspected: the icons have to stay readable
 * through the marks that explain them.
 */
export interface IEquipmentGridPalette {
  /** Lattice lines. */
  line: string;
  /** A slot whose section declares an icon of its own, which the packing tools act on. */
  declared: string;
  /** A slot inferred from grid fields alone, drawn weaker because nothing authored says an icon is there. */
  probable: string;
  /** The region past the edge of the picture, which nothing can ever be drawn in. */
  outside: string;
  /** Outline of a rectangle reaching into that region. */
  outsideEdge: string;
  /** Cell under the pointer. */
  hover: string;
  /** Cell whose occupants are being shown. */
  selected: string;
  /** Outline of that cell. */
  selectedEdge: string;
}

/**
 * Reads the overlay's colours off the active theme.
 *
 * @param theme - Theme in force.
 * @returns What each part of the lattice is drawn in.
 */
export function toEquipmentGridPalette(theme: Theme): IEquipmentGridPalette {
  return {
    declared: alpha(theme.palette.primary.main, 0.3),
    hover: alpha(theme.palette.common.white, 0.28),
    line: alpha(theme.palette.common.black, 0.45),
    // Half the weight of a declared slot, and no more: on Anomaly every one of the 2,096 occupants is probable, so a
    // treatment faint enough to read as absent would wash out the whole sheet.
    probable: alpha(theme.palette.primary.main, 0.15),
    outside: alpha(theme.palette.error.main, 0.2),
    outsideEdge: alpha(theme.palette.error.light, 0.9),
    selected: alpha(theme.palette.primary.main, 0.55),
    selectedEdge: alpha(theme.palette.primary.light, 0.95),
  };
}

/**
 * How strongly one occupant's slot is shaded.
 *
 * @param palette - Colours in force.
 * @param occupant - Section occupying the slot.
 * @returns The fill that says whether an icon there was authored or inferred.
 */
export function toClaimFill(palette: IEquipmentGridPalette, occupant: EquipmentSlotOccupant): string {
  return occupant.claim === EEquipmentSlotClaim.DECLARED ? palette.declared : palette.probable;
}
