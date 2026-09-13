// Auto-generated rust bindings. Do not edit it manually.

import {
  DialogsOpenRequest,
  DialogsReadRequest,
  SessionId,
  SessionRestore,
  SessionSnapshot,
} from "@/core/bindings/types/xrf-app";
import { DialogDescriptor, DialogProjectDescriptor, DialogProjectMode } from "@/core/bindings/types/xrf-dialog";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";
import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";

/** Commands */
export const dialogsCommands = {
  /** Releases only the committed and pending openings owned by the closing frontend. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:dialogs|close_project", { sessionIds }),
  /**
   * Report which layout roots looks like, for the open form to preselect.
   *
   * Advisory: `open_project` obeys whatever layout mode it is given, because the two layouts read and
   * write different files and a heuristic must not be what decides that. This mounts the roots to
   * answer, so it names one the same way the open does.
   */
  detectMode: (roots: XrayRoots) => __TAURI_INVOKE<DialogProjectMode>("plugin:dialogs|detect_mode", { roots }),
  /**
   * One dialog, with every phrase it declares.
   *
   * The project response carries only summaries — 502 dialogs' worth of phrases is a payload nobody
   * reads — so this is what a selection fetches. Served from the parsed project already in state, so
   * it costs a lookup rather than a read.
   *
   * Addressed by file and id together, because ids are not unique across a tree: a mod overlaying a
   * dialog keeps the original's id, and searching every file would silently answer with whichever copy
   * was read first.
   *
   * `language` picks which of the project's languages the phrase lines are resolved in, defaulting to
   * the first the text tree offers. Switching language is another call rather than a payload carrying
   * all of them: the index is already resident, so it costs a lookup, and one dialog in nine languages
   * would be nine times the bytes to display an eighth of it.
   */
  getDialog: (request: DialogsReadRequest) =>
    __TAURI_INVOKE<DialogDescriptor>("plugin:dialogs|get_dialog", { request }),
  getProject: () => __TAURI_INVOKE<SessionRestore<DialogProjectDescriptor>>("plugin:dialogs|get_project"),
  /**
   * Open a dialog tree.
   *
   * Two arguments, because opening answers two questions. `roots` is the shared vocabulary every
   * surface names roots with — ordered roots, each with its own mount mode — so an installation opens
   * as readily as a loose tree and a gamedata tree layers in front of one. `layout` is this domain's
   * own half: where inside those trees the dialogs and their text sit.
   *
   * The layout mode is obeyed, never re-derived: it decides which files a later save writes, so a guess
   * acted on here would decide what gets overwritten. `detect_mode` is what preselects it.
   */
  openProject: (request: DialogsOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<DialogProjectDescriptor>>("plugin:dialogs|open_project", { request }),
};
