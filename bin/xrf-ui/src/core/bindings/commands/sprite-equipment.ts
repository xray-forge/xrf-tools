// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import {
  EquipmentSpriteMetadata,
  PackSpriteRequest,
  SessionId,
  SessionRestore,
  SessionSnapshot,
  SpriteEquipmentOpenRequest,
} from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { PackEquipmentResult } from "@/core/bindings/types/xrf-texture";

/** Commands */
export const spriteEquipmentCommands = {
  closeSprite: (sessionIds: Array<SessionId>) =>
    __TAURI_INVOKE<null>("plugin:sprite-equipment|close_sprite", { sessionIds }),
  getSprite: () => __TAURI_INVOKE<SessionRestore<EquipmentSpriteMetadata>>("plugin:sprite-equipment|get_sprite"),
  openSprite: (request: SpriteEquipmentOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<EquipmentSpriteMetadata>>("plugin:sprite-equipment|open_sprite", { request }),
  reopenSprite: (sessionId: SessionId, openingId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<EquipmentSpriteMetadata>>("plugin:sprite-equipment|reopen_sprite", {
      sessionId,
      openingId,
    }),
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
