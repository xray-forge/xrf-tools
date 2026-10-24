// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  DialogsOpenRequest,
  DialogsReadRequest,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/ipc/types/xrf-app";
import { DialogDescriptor, DialogProjectDescriptor, DialogProjectMode } from "@/core/ipc/types/xrf-dialog";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const dialogsCommands = {
  /** Close the dialogs projects `session_ids` name, committed or opening; a closed one answers no read. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>("plugin:dialogs|close_project", { sessionIds }),
  /** Report which layout roots looks like, for the open form to preselect; on the pool, since it looks through the roots. */
  detectMode: (roots: XrayRoots) => __TAURI_INVOKE<DialogProjectMode>("plugin:dialogs|detect_mode", { roots }),
  /** One dialog, with every phrase it declares. */
  getDialog: (request: DialogsReadRequest) =>
    __TAURI_INVOKE<DialogDescriptor>("plugin:dialogs|get_dialog", { request }),
  /** Open a dialog tree, reading every dialog file and string table it holds on the pool. */
  openProject: (request: DialogsOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<DialogProjectDescriptor>>("plugin:dialogs|open_project", { request }),
  /**
   * The dialogs project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
   * reload.
   */
  restoreProject: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<DialogProjectDescriptor>>("plugin:dialogs|restore_project", { tool }),
};
