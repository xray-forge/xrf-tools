// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import {
  DialogFileDescriptor,
  DialogFinding,
  DialogProjectDescriptor,
  DialogProjectMode,
} from "@/core/bindings/types/xrf-dialog";

/** Commands */
export const dialogsCommands = {
  closeProject: () => __TAURI_INVOKE<null>("plugin:dialogs|close_project"),
  /**
   * Report which layout a directory looks like, for the open form to preselect.
   *
   * Advisory: `open_project` obeys whatever mode it is given, because the two layouts read and write
   * different files and a heuristic must not be what decides that.
   */
  detectMode: (path: string) => __TAURI_INVOKE<DialogProjectMode>("plugin:dialogs|detect_mode", { path }),
  /**
   * The open project, described again rather than cached.
   *
   * Provisioning asks the backend what is open, so a reload restores the session; the descriptor is
   * derived from the project on demand because the project is what state owns.
   */
  getProject: () =>
    __TAURI_INVOKE<{
      mode: DialogProjectMode;
      root: string;
      dialogsRoot: string;
      translationsRoot: string;
      /** Files keyed by their path relative to the dialogs root, in discovery order. */
      files: { [key in string]: DialogFileDescriptor };
      findings: Array<DialogFinding>;
    } | null>("plugin:dialogs|get_project"),
  /**
   * Open a dialog tree, in the layout the caller names.
   *
   * The mode is obeyed, never re-derived: it decides which files a later save writes, so a guess
   * acted on here would decide what gets overwritten. `detect_mode` is what preselects it.
   *
   * Both path overrides are optional and each stands in for one root. Source mode needs them least
   * often and gamedata mode most: a mod that keeps its dialogs somewhere the layout does not predict
   * is otherwise unopenable.
   */
  openProject: (path: string, mode: DialogProjectMode, dialogsPath: string | null, translationsPath: string | null) =>
    __TAURI_INVOKE<DialogProjectDescriptor>("plugin:dialogs|open_project", {
      path,
      mode,
      dialogsPath,
      translationsPath,
    }),
};
