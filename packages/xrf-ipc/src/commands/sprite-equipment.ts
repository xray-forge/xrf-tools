// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { SessionId, SessionRestore, SessionSnapshot } from "#/types/xrf-app-core";
import {
  EquipmentSpriteMetadata,
  SpriteEquipmentOpenRequest,
  SpriteEquipmentPackRequest,
} from "#/types/xrf-app-sprite-equipment";
import { JobProgress } from "#/types/xrf-job";
import { PackEquipmentResult } from "#/types/xrf-texture";

/** The wire name of every command of the `sprite-equipment` plugin. */
export enum ESpriteEquipmentCommand {
  CLOSE_SPRITE = "plugin:sprite-equipment|close_sprite",
  OPEN_SPRITE = "plugin:sprite-equipment|open_sprite",
  PACK_SPRITE = "plugin:sprite-equipment|pack_sprite",
  RELOAD_SPRITE = "plugin:sprite-equipment|reload_sprite",
  RESTORE_SPRITE = "plugin:sprite-equipment|restore_sprite",
}

/** Commands */
export const spriteEquipmentCommands = {
  /** Close the sheets `session_ids` name, committed or opening; a closed one answers no read. */
  closeSprite: (sessionIds: Array<SessionId>) =>
    __TAURI_INVOKE<void>(ESpriteEquipmentCommand.CLOSE_SPRITE, { sessionIds }),
  /** Open an equipment sheet with the configuration naming what sits on it, read on the pool. */
  openSprite: (request: SpriteEquipmentOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<EquipmentSpriteMetadata>>(ESpriteEquipmentCommand.OPEN_SPRITE, { request }),
  /** Draw every declared inventory icon into one equipment sprite sheet. */
  packSprite: (request: SpriteEquipmentPackRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<PackEquipmentResult>(ESpriteEquipmentCommand.PACK_SPRITE, { request, jobId, progress }),
  /**
   * Read the sheet `previous_id` holds again, as it was asked for, into the new handle `session_id`; the previous handle
   * is left as it is, for the page to close once the reload is shown, and a failed reload leaves it untouched.
   */
  reloadSprite: (sessionId: SessionId, previousId: SessionId) =>
    __TAURI_INVOKE<SessionSnapshot<EquipmentSpriteMetadata>>(ESpriteEquipmentCommand.RELOAD_SPRITE, {
      sessionId,
      previousId,
    }),
  /** The sheet `tool` holds in the asking window, the newest it opened, for a page attaching to it after a reload. */
  restoreSprite: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<EquipmentSpriteMetadata>>(ESpriteEquipmentCommand.RESTORE_SPRITE, { tool }),
};

/** What each command of the `sprite-equipment` plugin takes and resolves to, keyed by its wire name. */
export interface ISpriteEquipmentCommandSignatures {
  [ESpriteEquipmentCommand.CLOSE_SPRITE]: IIpcCommandSignature<
    typeof spriteEquipmentCommands.closeSprite,
    ["sessionIds"]
  >;
  [ESpriteEquipmentCommand.OPEN_SPRITE]: IIpcCommandSignature<typeof spriteEquipmentCommands.openSprite, ["request"]>;
  [ESpriteEquipmentCommand.PACK_SPRITE]: IIpcCommandSignature<
    typeof spriteEquipmentCommands.packSprite,
    ["request", "jobId", "progress"]
  >;
  [ESpriteEquipmentCommand.RELOAD_SPRITE]: IIpcCommandSignature<
    typeof spriteEquipmentCommands.reloadSprite,
    ["sessionId", "previousId"]
  >;
  [ESpriteEquipmentCommand.RESTORE_SPRITE]: IIpcCommandSignature<
    typeof spriteEquipmentCommands.restoreSprite,
    ["tool"]
  >;
}
