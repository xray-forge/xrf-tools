// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import { SessionId, SessionRestore, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { ExportSourceContent, ExportsProject } from "@/core/ipc/types/xrf-export";

/** Commands */
export const exportsCommands = {
  /** Releases only the committed and pending openings owned by the closing frontend. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:exports|close_project", { sessionIds }),
  exportManifest: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<null>("plugin:exports|export_manifest", { sessionId, path }),
  openProject: (sessionId: SessionId, projectPath: string) =>
    __TAURI_INVOKE<SessionSnapshot<ExportsProject>>("plugin:exports|open_project", { sessionId, projectPath }),
  getProject: () => __TAURI_INVOKE<SessionRestore<ExportsProject>>("plugin:exports|get_project"),
  getSource: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<ExportSourceContent>("plugin:exports|get_source", { sessionId, name }),
};
