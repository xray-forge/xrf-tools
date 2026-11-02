// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { SessionId, SessionRestore, SessionSnapshot } from "#/types/xrf-app-core";
import { DialogsOpenRequest, DialogsReadRequest } from "#/types/xrf-app-dialogs";
import { DialogDescriptor, DialogProjectDescriptor, DialogProjectMode } from "#/types/xrf-dialog";
import { XrayRoots } from "#/types/xrf-vfs";

/** The wire name of every command of the `dialogs` plugin. */
export enum EDialogsCommand {
  CLOSE_PROJECT = "plugin:dialogs|close_project",
  DETECT_MODE = "plugin:dialogs|detect_mode",
  GET_DIALOG = "plugin:dialogs|get_dialog",
  OPEN_PROJECT = "plugin:dialogs|open_project",
  RESTORE_PROJECT = "plugin:dialogs|restore_project",
}

/** Commands */
export const dialogsCommands = {
  /** Close the dialogs projects `session_ids` name, committed or opening; a closed one answers no read. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(EDialogsCommand.CLOSE_PROJECT, { sessionIds }),
  /** Report which layout roots looks like, for the open form to preselect; on the pool, since it looks through the roots. */
  detectMode: (roots: XrayRoots) => __TAURI_INVOKE<DialogProjectMode>(EDialogsCommand.DETECT_MODE, { roots }),
  /** One dialog, with every phrase it declares. */
  getDialog: (request: DialogsReadRequest) => __TAURI_INVOKE<DialogDescriptor>(EDialogsCommand.GET_DIALOG, { request }),
  /** Open a dialog tree, reading every dialog file and string table it holds on the pool. */
  openProject: (request: DialogsOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<DialogProjectDescriptor>>(EDialogsCommand.OPEN_PROJECT, { request }),
  /**
   * The dialogs project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
   * reload.
   */
  restoreProject: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<DialogProjectDescriptor>>(EDialogsCommand.RESTORE_PROJECT, { tool }),
};

/** What each command of the `dialogs` plugin takes and resolves to, keyed by its wire name. */
export interface IDialogsCommandSignatures {
  [EDialogsCommand.CLOSE_PROJECT]: IIpcCommandSignature<typeof dialogsCommands.closeProject, ["sessionIds"]>;
  [EDialogsCommand.DETECT_MODE]: IIpcCommandSignature<typeof dialogsCommands.detectMode, ["roots"]>;
  [EDialogsCommand.GET_DIALOG]: IIpcCommandSignature<typeof dialogsCommands.getDialog, ["request"]>;
  [EDialogsCommand.OPEN_PROJECT]: IIpcCommandSignature<typeof dialogsCommands.openProject, ["request"]>;
  [EDialogsCommand.RESTORE_PROJECT]: IIpcCommandSignature<typeof dialogsCommands.restoreProject, ["tool"]>;
}
