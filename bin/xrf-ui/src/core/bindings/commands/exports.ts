// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { DocumentRestore, DocumentSessionId, DocumentSnapshot } from "@/core/bindings/types/xrf-app";
import { ExportSourceContent, ExportsProject } from "@/core/bindings/types/xrf-export";

/** Commands */
export const exportsCommands = {
  /** Releases only the committed and pending openings owned by the closing frontend. */
  closeProject: (sessionIds: Array<DocumentSessionId>) =>
    __TAURI_INVOKE<null>("plugin:exports|close_project", { sessionIds }),
  openProject: (sessionId: DocumentSessionId, projectPath: string) =>
    __TAURI_INVOKE<DocumentSnapshot<ExportsProject>>("plugin:exports|open_project", { sessionId, projectPath }),
  getProject: () => __TAURI_INVOKE<DocumentRestore<ExportsProject>>("plugin:exports|get_project"),
  getSource: (sessionId: DocumentSessionId, name: string) =>
    __TAURI_INVOKE<ExportSourceContent>("plugin:exports|get_source", { sessionId, name }),
};
