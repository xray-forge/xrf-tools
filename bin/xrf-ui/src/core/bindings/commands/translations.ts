// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE } from "@tauri-apps/api/core";

import { TranslationParseSummary, TranslationVerifySummary } from "@/core/bindings/types/xrf-app";
import {
  TranslationEdit,
  TranslationFile,
  TranslationFinding,
  TranslationProjectDescriptor,
  TranslationProjectMode,
} from "@/core/bindings/types/xrf-translation";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const translationsCommands = {
  closeProject: () => __TAURI_INVOKE<null>("plugin:translations|close_project"),
  /**
   * Report which layout roots look like, for the open form to preselect.
   *
   * Advisory: `open_project` obeys whatever mode it is given, because the two layouts read and write
   * different files and a heuristic must not be what decides that. This mounts the roots to answer, so
   * it names one the same way the open does.
   */
  detectMode: (roots: XrayRoots) =>
    __TAURI_INVOKE<TranslationProjectMode>("plugin:translations|detect_mode", { roots }),
  getProject: () =>
    __TAURI_INVOKE<{
      mode: TranslationProjectMode;
      /** The roots this project was opened over, echoed back so a follow-up read addresses the same trees. */
      roots: XrayRoots;
      /** Logical prefix the string tables were read from. */
      prefix: string;
      /** Every language the root offers, in discovery order. */
      languages: Array<string>;
      /**
       * The code page each language is written in, which is what limits the characters it can hold.
       *
       * Taken from the files themselves in gamedata mode, so a language XRF has never heard of still
       * reports the encoding its own declaration claims.
       */
      encodings: { [key in string]: string };
      /**
       * Whether every file this project holds is loose, so an editing session could save all of it.
       *
       * One flag rather than a tree of them, so a surface can say up front that a project opened over an
       * installation is read-only. Which particular file refuses is answered by its source's absent
       * physical path.
       */
      isEditable: boolean;
      /** Files keyed by the logical name the layout groups them under. */
      files: { [key in string]: TranslationFile };
      findings: Array<TranslationFinding>;
    } | null>("plugin:translations|get_project"),
  /**
   * Open a translations tree.
   *
   * `roots` is the shared vocabulary every surface names roots with, so an installation opens as
   * readily as a loose tree and a gamedata tree layers in front of one. The prefix is this layout's
   * own half — where inside those trees the string tables sit — and defaults to what the mode implies.
   */
  openProject: (roots: XrayRoots, mode: TranslationProjectMode, prefix: string | null) =>
    __TAURI_INVOKE<TranslationProjectDescriptor>("plugin:translations|open_project", { roots, mode, prefix }),
  /** Import one language's raw XML string tables into JSON sources. */
  parseProject: (
    roots: XrayRoots,
    language: string,
    prefix: string | null,
    outputDir: string,
    file: string | null,
    isOverwrite: boolean,
    isDryRun: boolean
  ) =>
    __TAURI_INVOKE<TranslationParseSummary>("plugin:translations|parse_project", {
      roots,
      language,
      prefix,
      outputDir,
      file,
      isOverwrite,
      isDryRun,
    }),
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
   */
  saveFile: (file: string, edits: { [key in string]: Array<TranslationEdit> }) =>
    __TAURI_INVOKE<TranslationProjectDescriptor>("plugin:translations|save_file", { file, edits }),
  /**
   * Report the first character a language cannot hold, or nothing when the value is writable.
   *
   * Checked here rather than in the interface because the answer depends on code page tables the
   * browser has no encoder for, and on what each language's own files declared. Called when a cell is
   * committed, so a mistake is reported where it was made instead of at the end of a batch save.
   */
  validateText: (language: string, text: string) =>
    __TAURI_INVOKE<string | null>("plugin:translations|validate_text", { language, text }),
  /**
   * Report which translations are missing from which languages.
   *
   * Reads only. Nothing here writes, so an installation is a legitimate subject rather than a refusal.
   */
  verifyProject: (roots: XrayRoots, prefix: string | null, language: string) =>
    __TAURI_INVOKE<TranslationVerifySummary>("plugin:translations|verify_project", { roots, prefix, language }),
};
