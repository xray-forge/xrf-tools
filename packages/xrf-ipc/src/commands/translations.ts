// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
import { SessionId, SessionRestore, SessionSnapshot } from "#/types/xrf-app-core";
import {
  TranslationSaveOutcome,
  TranslationsBuildRequest,
  TranslationsFormatRequest,
  TranslationsOpenRequest,
  TranslationsParseRequest,
  TranslationsVerifyRequest,
} from "#/types/xrf-app-translations";
import { JobProgress } from "#/types/xrf-job";
import {
  TranslationBuildResult,
  TranslationEdit,
  TranslationFormatResult,
  TranslationParseResult,
  TranslationProjectDescriptor,
  TranslationProjectMode,
  TranslationVerifyResult,
} from "#/types/xrf-translation";
import { XrayRoots } from "#/types/xrf-vfs";

/** The wire name of every command of the `translations` plugin. */
export enum ETranslationsCommand {
  BUILD_PROJECT = "plugin:translations|build_project",
  CHECK_PROJECT_FORMAT = "plugin:translations|check_project_format",
  CLOSE_PROJECT = "plugin:translations|close_project",
  DETECT_MODE = "plugin:translations|detect_mode",
  FORMAT_PROJECT = "plugin:translations|format_project",
  OPEN_PROJECT = "plugin:translations|open_project",
  PARSE_PROJECT = "plugin:translations|parse_project",
  RESTORE_PROJECT = "plugin:translations|restore_project",
  SAVE_FILE = "plugin:translations|save_file",
  VALIDATE_TEXT = "plugin:translations|validate_text",
  VERIFY_PROJECT = "plugin:translations|verify_project",
}

/** Commands */
export const translationsCommands = {
  /** Compile translation sources into per-language string tables, one build at a time. */
  buildProject: (request: TranslationsBuildRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationBuildResult>(ETranslationsCommand.BUILD_PROJECT, { request, jobId, progress }),
  /**
   * Report which JSON translation sources under a directory are not normalized, in the format's own lane so a check
   * never reads half a rewrite.
   */
  checkProjectFormat: (request: TranslationsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationFormatResult>(ETranslationsCommand.CHECK_PROJECT_FORMAT, { request, jobId, progress }),
  /**
   * Close the translations projects `session_ids` name, committed or opening; a closed one answers no read and takes no
   * save.
   */
  closeProject: (sessionIds: Array<SessionId>) =>
    __TAURI_INVOKE<void>(ETranslationsCommand.CLOSE_PROJECT, { sessionIds }),
  /** Report which layout roots look like, for the open form to preselect; on the pool, since it looks through the roots. */
  detectMode: (roots: XrayRoots) => __TAURI_INVOKE<TranslationProjectMode>(ETranslationsCommand.DETECT_MODE, { roots }),
  /** Normalize the JSON translation sources under a directory, refused while an open project holds any of them. */
  formatProject: (request: TranslationsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationFormatResult>(ETranslationsCommand.FORMAT_PROJECT, { request, jobId, progress }),
  /**
   * Open a translations tree through the shared roots, read on the pool, recording what it reads so a commit to any of
   * those files reads the project again.
   */
  openProject: (request: TranslationsOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<TranslationProjectDescriptor>>(ETranslationsCommand.OPEN_PROJECT, { request }),
  /** Import one language's raw XML string tables into JSON sources, one import at a time. */
  parseProject: (request: TranslationsParseRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationParseResult>(ETranslationsCommand.PARSE_PROJECT, { request, jobId, progress }),
  /**
   * The translations project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
   * reload.
   */
  restoreProject: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<TranslationProjectDescriptor>>(ETranslationsCommand.RESTORE_PROJECT, { tool }),
  /** Write one logical file's pending edits, grouped by the language each belongs to, into the project `session_id` holds. */
  saveFile: (sessionId: SessionId, file: string, edits: { [key in string]: Array<TranslationEdit> }) =>
    __TAURI_INVOKE<TranslationSaveOutcome>(ETranslationsCommand.SAVE_FILE, { sessionId, file, edits }),
  /** Report the first character a language cannot hold, or nothing when the value is writable. */
  validateText: (sessionId: SessionId, language: string, text: string) =>
    __TAURI_INVOKE<string | null>(ETranslationsCommand.VALIDATE_TEXT, { sessionId, language, text }),
  /** Report which translations are missing from which languages, one check at a time. */
  verifyProject: (request: TranslationsVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationVerifyResult>(ETranslationsCommand.VERIFY_PROJECT, { request, jobId, progress }),
};

/** What each command of the `translations` plugin takes and resolves to, keyed by its wire name. */
export interface ITranslationsCommandSignatures {
  [ETranslationsCommand.BUILD_PROJECT]: IIpcCommandSignature<
    typeof translationsCommands.buildProject,
    ["request", "jobId", "progress"]
  >;
  [ETranslationsCommand.CHECK_PROJECT_FORMAT]: IIpcCommandSignature<
    typeof translationsCommands.checkProjectFormat,
    ["request", "jobId", "progress"]
  >;
  [ETranslationsCommand.CLOSE_PROJECT]: IIpcCommandSignature<typeof translationsCommands.closeProject, ["sessionIds"]>;
  [ETranslationsCommand.DETECT_MODE]: IIpcCommandSignature<typeof translationsCommands.detectMode, ["roots"]>;
  [ETranslationsCommand.FORMAT_PROJECT]: IIpcCommandSignature<
    typeof translationsCommands.formatProject,
    ["request", "jobId", "progress"]
  >;
  [ETranslationsCommand.OPEN_PROJECT]: IIpcCommandSignature<typeof translationsCommands.openProject, ["request"]>;
  [ETranslationsCommand.PARSE_PROJECT]: IIpcCommandSignature<
    typeof translationsCommands.parseProject,
    ["request", "jobId", "progress"]
  >;
  [ETranslationsCommand.RESTORE_PROJECT]: IIpcCommandSignature<typeof translationsCommands.restoreProject, ["tool"]>;
  [ETranslationsCommand.SAVE_FILE]: IIpcCommandSignature<
    typeof translationsCommands.saveFile,
    ["sessionId", "file", "edits"]
  >;
  [ETranslationsCommand.VALIDATE_TEXT]: IIpcCommandSignature<
    typeof translationsCommands.validateText,
    ["sessionId", "language", "text"]
  >;
  [ETranslationsCommand.VERIFY_PROJECT]: IIpcCommandSignature<
    typeof translationsCommands.verifyProject,
    ["request", "jobId", "progress"]
  >;
}
