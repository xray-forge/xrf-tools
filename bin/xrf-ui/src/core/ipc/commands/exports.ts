// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { SessionId, SessionRestore, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { ExportSourceContent, ExportsProject } from "@/core/ipc/types/xrf-export";

/** Commands */
export const exportsCommands = {
  /** Close the exports projects `session_ids` name, committed or opening; a closed one answers no read. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>("plugin:exports|close_project", { sessionIds }),
  /** Write the manifest of every extern the project `session_id` holds to `path`, in the format its extension names. */
  exportManifest: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<null>("plugin:exports|export_manifest", { sessionId, path }),
  /** Read the source the extern `name` is declared in, from the project `session_id` holds. */
  getSource: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<ExportSourceContent>("plugin:exports|get_source", { sessionId, name }),
  /**
   * Open the scripts project at `project_path` and parse every extern it declares, answering the whole project, which
   * the page lists in full.
   */
  openProject: (sessionId: SessionId, projectPath: string) =>
    __TAURI_INVOKE<SessionSnapshot<ExportsProject>>("plugin:exports|open_project", { sessionId, projectPath }),
  /**
   * The exports project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
   * reload.
   */
  restoreProject: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<ExportsProject>>("plugin:exports|restore_project", { tool }),
};
