// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  EquipmentSpriteMetadata,
  SessionId,
  SessionRestore,
  SessionSnapshot,
  SpriteEquipmentOpenRequest,
  SpriteEquipmentPackRequest,
} from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import { PackEquipmentResult } from "@/core/ipc/types/xrf-texture";

/** Commands */
export const spriteEquipmentCommands = {
  /** Close the sheets `session_ids` name, committed or opening; a closed one answers no read. */
  closeSprite: (sessionIds: Array<SessionId>) =>
    __TAURI_INVOKE<void>("plugin:sprite-equipment|close_sprite", { sessionIds }),
  /** Open an equipment sheet with the configuration naming what sits on it, read on the pool. */
  openSprite: (request: SpriteEquipmentOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<EquipmentSpriteMetadata>>("plugin:sprite-equipment|open_sprite", { request }),
  /** Draw every declared inventory icon into one equipment sprite sheet. */
  packSprite: (request: SpriteEquipmentPackRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<PackEquipmentResult>("plugin:sprite-equipment|pack_sprite", { request, jobId, progress }),
  /**
   * Read the sheet `previous_id` holds again, as it was asked for, into the new handle `session_id`; the previous handle
   * is left as it is, for the page to close once the reload is shown, and a failed reload leaves it untouched.
   */
  reloadSprite: (sessionId: SessionId, previousId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<EquipmentSpriteMetadata>>("plugin:sprite-equipment|reload_sprite", {
      sessionId,
      previousId,
    }),
  /** The sheet `tool` holds in the asking window, the newest it opened, for a page attaching to it after a reload. */
  restoreSprite: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<EquipmentSpriteMetadata>>("plugin:sprite-equipment|restore_sprite", { tool }),
};
