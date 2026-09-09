// Auto-generated rust bindings. Do not edit it manually.

import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core";

import {
  ConfigsDocument,
  ConfigsFormatRequest,
  ConfigsOpenRequest,
  ConfigsProjectDescriptor,
  ConfigsReadDocumentRequest,
  ConfigsReadSectionsRequest,
  ConfigsResolvedRequest,
  ConfigsSessionId,
  ConfigsVerifyRequest,
} from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { LtxProjectFormatResult, LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { LtxInventory, LtxResolvedIndex, LtxResolvedSection } from "@/core/bindings/types/xrf-ltx-inspect";
import { XrayRoots } from "@/core/bindings/types/xrf-vfs";

/** Commands */
export const configsCommands = {
  /**
   * Report which LTX configs roots exposes are misformatted.
   *
   * Reads archived configs too. Shares the formatter's exclusion group while retaining a separate job kind because
   * checking reports findings without rewriting files.
   */
  checkDirectoryFormat: (request: ConfigsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|check_directory_format", { request, jobId, progress }),
  /**
   * Closes the configs project, releasing its mounts and whatever it had resolved.
   *
   * Reissues the session identity, so a read already in flight cannot commit against the project that follows.
   */
  closeProject: () => __TAURI_INVOKE<null>("plugin:configs|close_project"),
  /**
   * Rewrite the LTX configs roots exposes.
   *
   * Writing needs a file, so this refuses a project holding archived winners — the refusal comes from
   * `xrf-ltx` itself. Formatting an installation is therefore a legitimate refusal, not a gap.
   *
   * Holds the roots exclusively for the whole run, so a second request over the same set is refused rather than allowed
   * to rewrite the files this one is walking. A cancelled run leaves the files it had already formatted formatted and
   * the rest untouched: each file is rewritten through a staged replace, so nothing is half-written and running it
   * again resolves the difference.
   */
  formatDirectory: (request: ConfigsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|format_directory", { request, jobId, progress }),
  /**
   * What configs project is open, for a frontend restoring itself after a reload.
   *
   * Inline rather than blocking: it hands back a handle on what is already held and reads nothing.
   */
  getProject: () =>
    __TAURI_INVOKE<{
      /** Identity every later read is addressed by. */
      sessionId: ConfigsSessionId;
      /** The trees this project searched, as the backend resolved them, so a reload restores the same open. */
      roots: XrayRoots;
      /** Scope inside those trees, or nothing for all of them. */
      prefix: string | null;
      /** Whether configs resolve under the Monolith/Anomaly patch dialect. */
      isDltx: boolean;
      /** Host path the project reports itself at, for a crumb that names something a person recognises. */
      root: string;
      /** Every config the project holds, and what each one is to it. */
      inventory: LtxInventory;
      /**
       * Section schemes the project's `*.scheme.ltx` files declare, by name.
       *
       * Sent once with the open rather than per document: a tree declares tens of them and every structure read joins
       * against the same set.
       */
      declaredSchemes: Array<string>;
    } | null>("plugin:configs|get_project"),
  /**
   * Lists every section one entry point resolves to, named and counted.
   *
   * The index, not the bodies: a vanilla `system.ltx` resolves to 23,500 sections holding 293,000 fields, and sending
   * those together would be a message of tens of megabytes for a screen showing forty lines. What travels is enough to
   * lay the document out - how many fields each section has, so the view knows its own height - and bodies are asked
   * for a page at a time as they scroll into view.
   */
  listResolvedSections: (request: ConfigsResolvedRequest) =>
    __TAURI_INVOKE<LtxResolvedIndex>("plugin:configs|list_resolved_sections", { request }),
  /** Opens a configs project for browsing, and lists what it holds. */
  openProject: (request: ConfigsOpenRequest) =>
    __TAURI_INVOKE<ConfigsProjectDescriptor>("plugin:configs|open_project", { request }),
  /** Reads one config as the authored view renders it: its lines, and what only the parser knows about them. */
  readDocument: (request: ConfigsReadDocumentRequest) =>
    __TAURI_INVOKE<ConfigsDocument>("plugin:configs|read_document", { request }),
  /**
   * Reads the bodies of the named sections of one entry point.
   *
   * Addressed by name rather than by offset, so a page is always whole sections and a filter applied on one side never
   * has to be mirrored on the other. A name the root does not hold is skipped rather than refused: a page request races
   * an index the caller may have fetched before a reopen.
   *
   * Inline rather than blocking: the root is already resolved by the time anything can ask for a page, so this reads
   * fields out of a map and the headers of at most a few declaring configs.
   */
  readResolvedSections: (request: ConfigsReadSectionsRequest) =>
    __TAURI_INVOKE<Array<LtxResolvedSection>>("plugin:configs|read_resolved_sections", { request }),
  /** Verifies LTX configs through the VFS, including archived files. */
  verifyDirectory: (request: ConfigsVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectVerifyResult>("plugin:configs|verify_directory", { request, jobId, progress }),
};
