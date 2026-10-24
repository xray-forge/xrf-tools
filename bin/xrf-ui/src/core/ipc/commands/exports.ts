// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { IIpcCommandSignature } from "@/core/ipc/signature";
import { SessionId, SessionRestore, SessionSnapshot } from "@/core/ipc/types/xrf-app-core";
import { ExportSourceContent, ExportsProject } from "@/core/ipc/types/xrf-export";

/** The wire name of every command of the `exports` plugin. */
export enum EExportsCommand {
  CLOSE_PROJECT = "plugin:exports|close_project",
  EXPORT_MANIFEST = "plugin:exports|export_manifest",
  GET_SOURCE = "plugin:exports|get_source",
  OPEN_PROJECT = "plugin:exports|open_project",
  RESTORE_PROJECT = "plugin:exports|restore_project",
}

/** Commands */
export const exportsCommands = {
  /** Close the exports projects `session_ids` name, committed or opening; a closed one answers no read. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(EExportsCommand.CLOSE_PROJECT, { sessionIds }),
  /** Write the manifest of every extern the project `session_id` holds to `path`, in the format its extension names. */
  exportManifest: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<null>(EExportsCommand.EXPORT_MANIFEST, { sessionId, path }),
  /** Read the source the extern `name` is declared in, from the project `session_id` holds. */
  getSource: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<ExportSourceContent>(EExportsCommand.GET_SOURCE, { sessionId, name }),
  /**
   * Open the scripts project at `project_path` and parse every extern it declares, answering the whole project, which
   * the page lists in full.
   */
  openProject: (sessionId: SessionId, projectPath: string) =>
    __TAURI_INVOKE<SessionSnapshot<ExportsProject>>(EExportsCommand.OPEN_PROJECT, { sessionId, projectPath }),
  /**
   * The exports project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
   * reload.
   */
  restoreProject: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<ExportsProject>>(EExportsCommand.RESTORE_PROJECT, { tool }),
};

/** What each command of the `exports` plugin takes and resolves to, keyed by its wire name. */
export interface IExportsCommandSignatures {
  [EExportsCommand.CLOSE_PROJECT]: IIpcCommandSignature<typeof exportsCommands.closeProject, ["sessionIds"]>;
  [EExportsCommand.EXPORT_MANIFEST]: IIpcCommandSignature<typeof exportsCommands.exportManifest, ["sessionId", "path"]>;
  [EExportsCommand.GET_SOURCE]: IIpcCommandSignature<typeof exportsCommands.getSource, ["sessionId", "name"]>;
  [EExportsCommand.OPEN_PROJECT]: IIpcCommandSignature<
    typeof exportsCommands.openProject,
    ["sessionId", "projectPath"]
  >;
  [EExportsCommand.RESTORE_PROJECT]: IIpcCommandSignature<typeof exportsCommands.restoreProject, ["tool"]>;
}
