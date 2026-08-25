// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import {
  DialogDescriptor,
  DialogFileDescriptor,
  DialogFinding,
  DialogProjectDescriptor,
  DialogProjectMode,
} from "@/core/bindings/types/xrf-dialog";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const dialogsCommands = {
  closeProject: () => __TAURI_INVOKE<null>("plugin:dialogs|close_project"),
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
   */
  getDialog: (logicalPath: string, id: string) =>
    __TAURI_INVOKE<DialogDescriptor>("plugin:dialogs|get_dialog", { logicalPath, id }),
  /**
   * The open project, described again rather than cached.
   *
   * Provisioning asks the backend what is open, so a reload restores the session; the descriptor is
   * derived from the project on demand because the project is what state owns.
   */
  getProject: () =>
    __TAURI_INVOKE<{
      mode: DialogProjectMode;
      /** The roots this project was opened over, echoed back so a follow-up read addresses the same trees. */
      roots: XrayRoots;
      /** Logical prefix the dialogs were read from. */
      dialogsPrefix: string;
      /** Logical prefix dialog text is read from. */
      translationsPrefix: string;
      /** Whether every file the project holds is loose, so an editing session could save all of it. */
      isEditable: boolean;
      /** Files keyed by their logical path, in logical-path order. */
      files: { [key in string]: DialogFileDescriptor };
      findings: Array<DialogFinding>;
    } | null>("plugin:dialogs|get_project"),
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
  openProject: (
    roots: XrayRoots,
    mode: DialogProjectMode,
    dialogsPrefix: string | null,
    translationsPrefix: string | null
  ) =>
    __TAURI_INVOKE<DialogProjectDescriptor>("plugin:dialogs|open_project", {
      roots,
      mode,
      dialogsPrefix,
      translationsPrefix,
    }),
};
