// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import {
  DocumentRestore,
  DocumentSessionId,
  DocumentSnapshot,
  TranslationBuildRequest,
  TranslationBuildSummary,
  TranslationParseRequest,
  TranslationParseSummary,
  TranslationSaveOutcome,
  TranslationsFormatRequest,
  TranslationsOpenRequest,
  TranslationsVerifyRequest,
  TranslationVerifySummary,
} from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import {
  TranslationEdit,
  TranslationFormatResult,
  TranslationProjectDescriptor,
  TranslationProjectMode,
} from "@/core/bindings/types/xrf-translation";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const translationsCommands = {
  /**
   * Compile translation sources into per-language string tables.
   *
   * `roots` names where the sources are read from, through the VFS, so a tree layered over an
   * installation compiles what the engine would actually load. `outputDir` is a plain host directory,
   * because a string table is a file and a `.db` volume has nowhere to put one.
   *
   * Refuses an output directory inside any of the source roots before writing anything.
   */
  buildProject: (request: TranslationBuildRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationBuildSummary>("plugin:translations|build_project", { request, jobId, progress }),
  /**
   * Report which JSON translation sources under a directory are not normalized.
   *
   * Shares the formatter's exclusion group. Open editor sessions are allowed because checking does not rewrite files
   * or make their buffers stale; the separate job kind preserves that distinction in the reported outcome.
   */
  checkProjectFormat: (request: TranslationsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationFormatResult>("plugin:translations|check_project_format", { request, jobId, progress }),
  closeProject: (sessionIds: Array<DocumentSessionId>) =>
    __TAURI_INVOKE<null>("plugin:translations|close_project", { sessionIds }),
  /**
   * Report which layout roots look like, for the open form to preselect.
   *
   * Advisory: `open_project` obeys whatever mode it is given, because the two layouts read and write
   * different files and a heuristic must not be what decides that. This mounts the roots to answer, so
   * it names one the same way the open does.
   */
  detectMode: (roots: XrayRoots) =>
    __TAURI_INVOKE<TranslationProjectMode>("plugin:translations|detect_mode", { roots }),
  /**
   * Normalize the JSON translation sources under a directory.
   *
   * A host directory rather than mounted roots, because this rewrites its sources in place and there is nowhere to
   * put a file inside an archive volume. `configs format_directory` takes roots because an LTX project is a VFS
   * notion — winning configs, an include graph, archived entries it declines — and none of that applies to flat JSON.
   *
   * Holds the directory exclusively for the whole run, under the same lease a build and an import take, so a second
   * writer over the same tree is refused rather than allowed to read files this one is midway through replacing. A
   * cancelled run leaves the sources it had already formatted formatted and the rest untouched: each is rewritten
   * through a staged replace, so nothing is half-written and running it again resolves the difference.
   */
  formatProject: (request: TranslationsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationFormatResult>("plugin:translations|format_project", { request, jobId, progress }),
  getProject: () => __TAURI_INVOKE<DocumentRestore<TranslationProjectDescriptor>>("plugin:translations|get_project"),
  /**
   * Open a translations tree.
   *
   * `roots` is the shared vocabulary every surface names roots with, so an installation opens as
   * readily as a loose tree and a gamedata tree layers in front of one. The prefix is this layout's
   * own half — where inside those trees the string tables sit — and defaults to what the mode implies.
   */
  openProject: (request: TranslationsOpenRequest) =>
    __TAURI_INVOKE<DocumentSnapshot<TranslationProjectDescriptor>>("plugin:translations|open_project", { request }),
  /** Import one language's raw XML string tables into JSON sources. */
  parseProject: (request: TranslationParseRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationParseSummary>("plugin:translations|parse_project", { request, jobId, progress }),
  /**
   * Write one logical file's pending edits, grouped by the language each belongs to.
   *
   * A logical file is several files on disk in gamedata mode, one per language, so the edits arrive
   * keyed by language and each group goes to its own source. The sources come from the open project
   * rather than from the caller, so a save can only ever touch files this project actually read.
   *
   * Each one is re-resolved through the VFS before it is written. The descriptor's own path is portable
   * and therefore lossy — a display form — and using it as a write address is how an edit lands
   * somewhere that is not the file. The mount answers with the real path and with what wins *now*.
   *
   * A language served out of an archive is refused by name rather than skipped, because a save that
   * silently drops one language's edits looks identical to one that succeeded.
   *
   * Answers `stale` when another project was opened or the project was closed while the edits were being written. The
   * edits are on disk in either case; what a stale answer withholds is the refreshed tree, which belongs to a project
   * the application is no longer showing.
   */
  saveFile: (sessionId: DocumentSessionId, file: string, edits: { [key in string]: Array<TranslationEdit> }) =>
    __TAURI_INVOKE<TranslationSaveOutcome>("plugin:translations|save_file", { sessionId, file, edits }),
  /**
   * Report the first character a language cannot hold, or nothing when the value is writable.
   *
   * Checked here rather than in the interface because the answer depends on code page tables the
   * browser has no encoder for, and on what each language's own files declared. Called when a cell is
   * committed, so a mistake is reported where it was made instead of at the end of a batch save.
   */
  validateText: (sessionId: DocumentSessionId, language: string, text: string) =>
    __TAURI_INVOKE<string | null>("plugin:translations|validate_text", { sessionId, language, text }),
  /**
   * Report which translations are missing from which languages.
   *
   * Reads only. Nothing here writes, so an installation is a legitimate subject rather than a refusal.
   */
  verifyProject: (request: TranslationsVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<TranslationVerifySummary>("plugin:translations|verify_project", { request, jobId, progress }),
};
