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
  /** Releases only the committed and pending openings owned by the closing frontend. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:dialogs|close_project", { sessionIds }),
  /** Report which layout roots looks like, for the open form to preselect. */
  detectMode: (roots: XrayRoots) => __TAURI_INVOKE<DialogProjectMode>("plugin:dialogs|detect_mode", { roots }),
  /** One dialog, with every phrase it declares. */
  getDialog: (request: DialogsReadRequest) =>
    __TAURI_INVOKE<DialogDescriptor>("plugin:dialogs|get_dialog", { request }),
  getProject: () => __TAURI_INVOKE<SessionRestore<DialogProjectDescriptor>>("plugin:dialogs|get_project"),
  /** Open a dialog tree. */
  openProject: (request: DialogsOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<DialogProjectDescriptor>>("plugin:dialogs|open_project", { request }),
};
