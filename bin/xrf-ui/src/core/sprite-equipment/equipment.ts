import { PackEquipmentResult } from "@/core/bindings/types/xrf-texture";

/** A rectangle in the packed equipment sprite, measured in grid cells. */
export interface IEquipmentSectionDescriptor {
  /** Configuration section that owns the rectangle. */
  section: string;
  /** Rectangle width in cells. */
  w: number;
  /** Rectangle height in cells. */
  h: number;
  /** Left cell coordinate. */
  x: number;
  /** Top cell coordinate. */
  y: number;
}

/** Paths and section rectangles needed to inspect one equipment sprite. */
export interface IEquipmentSpriteMetadata {
  /** Path to the `system.ltx` that supplied the section definitions. */
  systemLtxPath: string;
  /** Path to the packed sprite. */
  path: string;
  /** Display name for the sprite. */
  name: string;
  /** Rectangles declared by the source configuration. */
  equipmentDescriptors: Array<IEquipmentSectionDescriptor>;
}

/** A selected sprite cell as `[row, column]`. */
export type TEquipmentCell = [number, number];

/**
 * What packing a sprite sheet produced.
 *
 * The generated mirror rather than a hand-written copy of it. The copy had drifted the moment the Rust result gained a
 * field: it silently lacked `outcome`, so nothing here could tell a finished run from a stopped one.
 */
export type IPackEquipmentResult = PackEquipmentResult;
