// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import { EquipmentSpriteMetadata, PackSpriteRequest } from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { InventorySpriteDescriptor, PackEquipmentResult } from "@/core/bindings/types/xrf-texture";

/** Commands */
export const spriteEquipmentCommands = {
  closeSprite: () => __TAURI_INVOKE<null>("plugin:sprite-equipment|close_sprite"),
  getSprite: () =>
    __TAURI_INVOKE<{
      path: string;
      name: string;
      systemLtxPath: string;
      /** Whether these descriptors came out of a DLTX-resolved config tree. */
      isDltx: boolean;
      equipmentDescriptors: Array<InventorySpriteDescriptor>;
    } | null>("plugin:sprite-equipment|get_sprite"),
  openSprite: (equipmentDdsPath: string, systemLtxPath: string, isDltx: boolean) =>
    __TAURI_INVOKE<EquipmentSpriteMetadata>("plugin:sprite-equipment|open_sprite", {
      equipmentDdsPath,
      systemLtxPath,
      isDltx,
    }),
  reopenSprite: () => __TAURI_INVOKE<EquipmentSpriteMetadata>("plugin:sprite-equipment|reopen_sprite"),
  /**
   * Draw every declared inventory icon into one equipment sprite sheet.
   *
   * Holds the sheet it writes exclusively, so a second request for the same output is refused rather than allowed to
   * race it. A cancelled run leaves nothing behind: the sheet is one image written once at the end, so stopping before
   * that point writes no file at all.
   */
  packSprite: (request: PackSpriteRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<PackEquipmentResult>("plugin:sprite-equipment|pack_sprite", { request, jobId, progress }),
};
