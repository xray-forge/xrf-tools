// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import {
  DialogFileDescriptor,
  DialogFinding,
  DialogProjectDescriptor,
  DialogProjectMode,
} from "@/core/bindings/types/xrf-dialog";
import { XrayWorldSpec } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const dialogsCommands = {
  closeProject: () => __TAURI_INVOKE<null>("plugin:dialogs|close_project"),
  /**
   * Report which layout a world looks like, for the open form to preselect.
   *
   * Advisory: `open_project` obeys whatever layout mode it is given, because the two layouts read and
   * write different files and a heuristic must not be what decides that. This mounts the world to
   * answer, so it names one the same way the open does.
   */
  detectMode: (world: XrayWorldSpec) => __TAURI_INVOKE<DialogProjectMode>("plugin:dialogs|detect_mode", { world }),
  /**
   * The open project, described again rather than cached.
   *
   * Provisioning asks the backend what is open, so a reload restores the session; the descriptor is
   * derived from the project on demand because the project is what state owns.
   */
  getProject: () =>
    __TAURI_INVOKE<{
      mode: DialogProjectMode;
      /** The world this project was opened over, echoed back so a follow-up read addresses the same trees. */
      world: XrayWorldSpec;
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
   * Two arguments, because opening answers two questions. `world` is the shared vocabulary every
   * surface names a world with — ordered roots, each with its own mount mode — so an installation opens
   * as readily as a loose tree and a gamedata tree layers in front of one. `layout` is this domain's
   * own half: where inside those trees the dialogs and their text sit.
   *
   * The layout mode is obeyed, never re-derived: it decides which files a later save writes, so a guess
   * acted on here would decide what gets overwritten. `detect_mode` is what preselects it.
   */
  openProject: (
    world: XrayWorldSpec,
    mode: DialogProjectMode,
    dialogsPrefix: string | null,
    translationsPrefix: string | null
  ) =>
    __TAURI_INVOKE<DialogProjectDescriptor>("plugin:dialogs|open_project", {
      world,
      mode,
      dialogsPrefix,
      translationsPrefix,
    }),
};
