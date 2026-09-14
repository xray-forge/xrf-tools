// Auto-generated rust bindings. Do not edit it manually.

import { JobOutcome } from "@/core/ipc/types/xrf-job";

/** How strongly a section claims the slot it names. */
export enum EEquipmentSlotClaim {
  /** The section resolves `$inventory_icon = true`, so the packing tools act on it. */
  DECLARED = "declared",
  /** The section positions itself on the grid but declares no icon of its own. */
  PROBABLE = "probable",
}

/** Every `EEquipmentSlotClaim` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type EquipmentSlotClaim = `${EEquipmentSlotClaim}`;

/** A section that occupies part of an equipment sheet, as the editor reads it. */
export type EquipmentSlotOccupant = {
  section: string;
  claim: EquipmentSlotClaim;
  /** Where the section reads its icon from, when it overrides the default of `<section>.dds` beside the source. */
  customIcon: string | null;
  /** Engine identity of the config whose header declared the section, where the resolution stamped one. */
  origin: string | null;
  x: number;
  y: number;
  w: number;
  h: number;
};

export type PackEquipmentResult = {
  /**
   * Whether the run drew every section or was stopped between them.
   *
   * The sheet is one image written once at the end, so a stopped run leaves nothing behind: the counts describe what
   * it had drawn in memory, and no file was replaced.
   */
  outcome: JobOutcome;
  duration: number;
  savedAt: string;
  savedWidth: number;
  savedHeight: number;
  packedCount: number;
  skippedCount: number;
};
