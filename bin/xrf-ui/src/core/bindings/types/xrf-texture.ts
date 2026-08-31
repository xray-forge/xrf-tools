// Auto-generated rust bindings. Do not edit it manually.

import { JobOutcome } from "@/core/bindings/types/xrf-job";

export type InventorySpriteDescriptor = {
  section: string;
  customIcon: string | null;
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
