// Auto-generated rust bindings. Do not edit it manually.

import { AssetSource, SessionId } from "#/types/xrf-app-core";
import { EquipmentSlotOccupant } from "#/types/xrf-texture";
import { XrayAsset, XrayRoots } from "#/types/xrf-vfs";

/** Packing loose inventory icons into an equipment sprite sheet. */
export const SPRITE_EQUIPMENT_JOB_PACK = "sprite-equipment.pack" as const;

/** Where an opened sheet turned out to be, and whether anything can write there. */
export type EquipmentSheetLocation = {
  /** The sheet as the roots located it, or nothing for a file opened by path outside any tree. */
  asset: XrayAsset | null;
  /** Absolute path of the sheet, or nothing when it lives inside an archive volume. */
  path: string | null;
  /** Where a write to the sheet would land. */
  writeTarget: string | null;
};

/** Every `kind` the `EquipmentSheetSource` union is told apart by, so a switch or a comparison names one. */
export enum EEquipmentSheetSource {
  /** A loose `.dds` on disk, named by its filesystem path. */
  FILE = "file",
  /** A sheet of the roots, named by its engine reference such as `ui\ui_icon_equipment`. */
  ASSET = "asset",
}

/** Where an equipment sheet is read from. */
export type EquipmentSheetSource =
  /** A loose `.dds` on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** A sheet of the roots, named by its engine reference such as `ui\ui_icon_equipment`. */
  | { kind: "asset"; reference: string };

/** Everything the editor is told about one opened sheet. */
export type EquipmentSpriteMetadata = {
  /** What this open was asked to read, so a reload repeats the request rather than a reconstruction of it. */
  open: EquipmentSpriteOpen;
  /** Which copy of the sheet this is, and whether anything can write to it. */
  location: EquipmentSheetLocation;
  /** Why the configuration was not read, when it was asked for and could not be. */
  configError: string | null;
  /** Every section occupying a slot on the sheet, in the order the configuration declares them. */
  occupants: Array<EquipmentSlotOccupant>;
};

/** Everything one opening of a sheet was asked to read. */
export type EquipmentSpriteOpen = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  sheet: EquipmentSheetSource;
  /**
   * The `system.ltx` naming what sits on the sheet, a file or an entry point of the roots, or nothing to open it
   * unannotated.
   */
  config: AssetSource | null;
  /** Whether to resolve that configuration with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/** The complete identity and inputs of one sprite opening. */
export type SpriteEquipmentOpenRequest = {
  sessionId: SessionId;
} & EquipmentSpriteOpen;

/** What an equipment sprite pack was asked to do. */
export type SpriteEquipmentPackRequest = {
  /** Directory of loose icons to draw from. */
  sourcePath: string;
  /** Sheet to write. */
  outputPath: string;
  /** `system.ltx` declaring which icons exist and where they sit. */
  systemLtxPath: string;
  /** Whether to resolve that config with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};
