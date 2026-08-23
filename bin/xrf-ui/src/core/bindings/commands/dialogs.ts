// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import {
  DialogFileDescriptor,
  DialogFinding,
  DialogProjectDescriptor,
  DialogProjectMode,
} from "@/core/bindings/types/xrf-dialog";
import { XrayMountMode } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const dialogsCommands = {
  closeProject: () => __TAURI_INVOKE<null>("plugin:dialogs|close_project"),
  /**
   * Report which layout a path looks like, for the open form to preselect.
   *
   * Advisory: `open_project` obeys whatever layout mode it is given, because the two read and write
   * different files and a heuristic must not be what decides that. This mounts the world to answer,
   * so it takes the same `source` vocabulary the open does.
   */
  detectMode: (path: string, source: XrayMountMode) =>
    __TAURI_INVOKE<DialogProjectMode>("plugin:dialogs|detect_mode", { path, source }),
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
   * Open a dialog tree, in the layout the caller names.
   *
   * `source` is the same `XrayMountMode` vocabulary every surface exposes, so an installation opens as
   * readily as a loose tree: on a real game the dialogs come out of `db\configs`, and a reader reaching
   * for the filesystem would report them absent.
   *
   * The layout mode is obeyed, never re-derived: it decides which files a later save writes, so a guess
   * acted on here would decide what gets overwritten. `detect_mode` is what preselects it.
   *
   * Both prefix overrides are optional and each stands in for one logical prefix, so a mod keeping its
   * dialogs somewhere the layout does not predict still opens.
   */
  openProject: (
    path: string,
    source: XrayMountMode,
    mode: DialogProjectMode,
    dialogsPrefix: string | null,
    translationsPrefix: string | null
  ) =>
    __TAURI_INVOKE<DialogProjectDescriptor>("plugin:dialogs|open_project", {
      path,
      source,
      mode,
      dialogsPrefix,
      translationsPrefix,
    }),
};
