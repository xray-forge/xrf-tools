import { EquipmentSheetSource } from "@/core/ipc/types/xrf-app";

/**
 * What to call a sheet in a message about it.
 *
 * @param sheet - Where the sheet was read from.
 * @returns Its path, or the engine reference the roots were asked for.
 */
export function describeEquipmentSheet(sheet: EquipmentSheetSource): string {
  return sheet.kind === "file" ? sheet.path : sheet.reference;
}
