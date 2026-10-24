// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  SessionId,
  SessionRestore,
  SessionSnapshot,
  TranslationSaveOutcome,
  TranslationsBuildRequest,
  TranslationsFormatRequest,
  TranslationsOpenRequest,
  TranslationsParseRequest,
  TranslationsVerifyRequest,
} from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import {
  TranslationBuildResult,
  TranslationEdit,
  TranslationFormatResult,
  TranslationParseResult,
  TranslationProjectDescriptor,
  TranslationProjectMode,
  TranslationVerifyResult,
} from "@/core/ipc/types/xrf-translation";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const translationsCommands = {
  /** Compile translation sources into per-language string tables, one build at a time. */
  buildProject: (request: TranslationsBuildRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationBuildResult>("plugin:translations|build_project", { request, jobId, progress }),
  /**
   * Report which JSON translation sources under a directory are not normalized, in the format's own lane so a check
   * never reads half a rewrite.
   */
  checkProjectFormat: (request: TranslationsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationFormatResult>("plugin:translations|check_project_format", { request, jobId, progress }),
  /**
   * Close the translations projects `session_ids` name, committed or opening; a closed one answers no read and takes no
   * save.
   */
  closeProject: (sessionIds: Array<SessionId>) =>
    __TAURI_INVOKE<void>("plugin:translations|close_project", { sessionIds }),
  /** Report which layout roots look like, for the open form to preselect; on the pool, since it looks through the roots. */
  detectMode: (roots: XrayRoots) =>
    __TAURI_INVOKE<TranslationProjectMode>("plugin:translations|detect_mode", { roots }),
  /** Normalize the JSON translation sources under a directory, refused while an open project holds any of them. */
  formatProject: (request: TranslationsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationFormatResult>("plugin:translations|format_project", { request, jobId, progress }),
  /**
   * Open a translations tree through the shared roots, read on the pool, recording what it reads so a commit to any of
   * those files reads the project again.
   */
  openProject: (request: TranslationsOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<TranslationProjectDescriptor>>("plugin:translations|open_project", { request }),
  /** Import one language's raw XML string tables into JSON sources, one import at a time. */
  parseProject: (request: TranslationsParseRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationParseResult>("plugin:translations|parse_project", { request, jobId, progress }),
  /**
   * The translations project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
   * reload.
   */
  restoreProject: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<TranslationProjectDescriptor>>("plugin:translations|restore_project", { tool }),
  /** Write one logical file's pending edits, grouped by the language each belongs to, into the project `session_id` holds. */
  saveFile: (sessionId: SessionId, file: string, edits: { [key in string]: Array<TranslationEdit> }) =>
    __TAURI_INVOKE<TranslationSaveOutcome>("plugin:translations|save_file", { sessionId, file, edits }),
  /** Report the first character a language cannot hold, or nothing when the value is writable. */
  validateText: (sessionId: SessionId, language: string, text: string) =>
    __TAURI_INVOKE<string | null>("plugin:translations|validate_text", { sessionId, language, text }),
  /** Report which translations are missing from which languages, one check at a time. */
  verifyProject: (request: TranslationsVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationVerifyResult>("plugin:translations|verify_project", { request, jobId, progress }),
};
