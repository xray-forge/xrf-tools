// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
import {
  SessionId,
  SessionRestore,
  SessionSnapshot,
  TranslationBuildRequest,
  TranslationBuildSummary,
  TranslationParseRequest,
  TranslationParseSummary,
  TranslationSaveOutcome,
  TranslationsFormatRequest,
  TranslationsOpenRequest,
  TranslationsVerifyRequest,
  TranslationVerifySummary,
} from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import {
  TranslationEdit,
  TranslationFormatResult,
  TranslationProjectDescriptor,
  TranslationProjectMode,
} from "@/core/ipc/types/xrf-translation";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const translationsCommands = {
  /** Compile translation sources into per-language string tables. */
  buildProject: (request: TranslationBuildRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationBuildSummary>("plugin:translations|build_project", { request, jobId, progress }),
  /** Report which JSON translation sources under a directory are not normalized. */
  checkProjectFormat: (request: TranslationsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationFormatResult>("plugin:translations|check_project_format", { request, jobId, progress }),
  closeProject: (sessionIds: Array<SessionId>) =>
    __TAURI_INVOKE<null>("plugin:translations|close_project", { sessionIds }),
  /** Report which layout roots look like, for the open form to preselect. */
  detectMode: (roots: XrayRoots) =>
    __TAURI_INVOKE<TranslationProjectMode>("plugin:translations|detect_mode", { roots }),
  /** Normalize the JSON translation sources under a directory. */
  formatProject: (request: TranslationsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationFormatResult>("plugin:translations|format_project", { request, jobId, progress }),
  getProject: () => __TAURI_INVOKE<SessionRestore<TranslationProjectDescriptor>>("plugin:translations|get_project"),
  /** Open a translations tree. */
  openProject: (request: TranslationsOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<TranslationProjectDescriptor>>("plugin:translations|open_project", { request }),
  /** Import one language's raw XML string tables into JSON sources. */
  parseProject: (request: TranslationParseRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationParseSummary>("plugin:translations|parse_project", { request, jobId, progress }),
  /** Write one logical file's pending edits, grouped by the language each belongs to. */
  saveFile: (sessionId: SessionId, file: string, edits: { [key in string]: Array<TranslationEdit> }) =>
    __TAURI_INVOKE<TranslationSaveOutcome>("plugin:translations|save_file", { sessionId, file, edits }),
  /** Report the first character a language cannot hold, or nothing when the value is writable. */
  validateText: (sessionId: SessionId, language: string, text: string) =>
    __TAURI_INVOKE<string | null>("plugin:translations|validate_text", { sessionId, language, text }),
  /** Report which translations are missing from which languages. */
  verifyProject: (request: TranslationsVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationVerifySummary>("plugin:translations|verify_project", { request, jobId, progress }),
};
