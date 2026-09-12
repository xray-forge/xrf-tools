// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { SessionId, SessionRestore, SessionSnapshot } from "@/core/bindings/types/xrf-app";
import { ExportSourceContent, ExportsProject } from "@/core/bindings/types/xrf-export";

/** Commands */
export const exportsCommands = {
  /** Releases only the committed and pending openings owned by the closing frontend. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:exports|close_project", { sessionIds }),
  /**
   * Write the open project's externs out as one of the manifests `xrf-cli externs export` publishes.
   *
   * The writer is chosen from the destination's extension, the way a packing configuration picks one, so a surface
   * offering three formats needs no fourth argument to say which it asked for.
   *
   * Rendered from the manifest the open parsed rather than from a fresh read of the tree, so the artifact describes
   * exactly the declarations on screen; refreshing is how a person asks for a later read of the sources.
   */
  exportManifest: (sessionId: SessionId, path: string) =>
    __TAURI_INVOKE<null>("plugin:exports|export_manifest", { sessionId, path }),
  openProject: (sessionId: SessionId, projectPath: string) =>
    __TAURI_INVOKE<SessionSnapshot<ExportsProject>>("plugin:exports|open_project", { sessionId, projectPath }),
  getProject: () => __TAURI_INVOKE<SessionRestore<ExportsProject>>("plugin:exports|get_project"),
  getSource: (sessionId: SessionId, name: string) =>
    __TAURI_INVOKE<ExportSourceContent>("plugin:exports|get_source", { sessionId, name }),
};
