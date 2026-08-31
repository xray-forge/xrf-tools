// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import { EquipmentSpriteMetadata } from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { InventorySpriteDescriptor, PackEquipmentResult } from "@/core/bindings/types/xrf-texture";

/** Commands */
export const equipmentIconsCommands = {
  closeSprite: () => __TAURI_INVOKE<null>("plugin:equipment-icons|close_sprite"),
  getSprite: () =>
    __TAURI_INVOKE<{
      path: string;
      name: string;
      systemLtxPath: string;
      equipmentDescriptors: Array<InventorySpriteDescriptor>;
    } | null>("plugin:equipment-icons|get_sprite"),
  openSprite: (equipmentDdsPath: string, systemLtxPath: string) =>
    __TAURI_INVOKE<EquipmentSpriteMetadata>("plugin:equipment-icons|open_sprite", { equipmentDdsPath, systemLtxPath }),
  reopenSprite: () => __TAURI_INVOKE<EquipmentSpriteMetadata>("plugin:equipment-icons|reopen_sprite"),
  /**
   * Draw every declared inventory icon into one equipment sprite sheet.
   *
   * Holds the sheet it writes exclusively, so a second request for the same output is refused rather than allowed to
   * race it. A cancelled run leaves nothing behind: the sheet is one image written once at the end, so stopping before
   * that point writes no file at all.
   */
  packSprite: (
    sourcePath: string,
    outputPath: string,
    systemLtxPath: string,
    jobId: string,
    progress: Channel<JobProgress>
  ) =>
    __TAURI_INVOKE<PackEquipmentResult>("plugin:equipment-icons|pack_sprite", {
      sourcePath,
      outputPath,
      systemLtxPath,
      jobId,
      progress,
    }),
};
