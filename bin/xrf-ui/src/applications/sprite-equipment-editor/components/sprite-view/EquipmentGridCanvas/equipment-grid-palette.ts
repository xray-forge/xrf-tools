import { alpha, Theme } from "@mui/material";

/**
 * What each part of the lattice is drawn in.
 */
export interface IEquipmentGridPalette {
  /** Lattice lines. */
  line: string;
  /** Cells a section claims. */
  occupied: string;
  /** The region past the edge of the picture. */
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
    hover: alpha(theme.palette.common.white, 0.28),
    line: alpha(theme.palette.common.black, 0.45),
    occupied: alpha(theme.palette.primary.main, 0.22),
    outside: alpha(theme.palette.warning.main, 0.2),
    outsideEdge: alpha(theme.palette.warning.light, 0.85),
    selected: alpha(theme.palette.primary.main, 0.55),
    selectedEdge: alpha(theme.palette.primary.light, 0.95),
  };
}
