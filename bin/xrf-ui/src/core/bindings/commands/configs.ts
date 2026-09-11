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
  ConfigsSectionRequest,
  ConfigsVerifyRequest,
  DocumentSessionId,
} from "@/core/bindings/types/xrf-app";
import { JobProgress } from "@/core/bindings/types/xrf-job";
import { LtxProjectFormatResult, LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import {
  LtxAnchoredFinding,
  LtxInventory,
  LtxResolvedIndex,
  LtxResolvedSection,
  LtxSchemeFieldReport,
} from "@/core/bindings/types/xrf-ltx-inspect";
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
  closeProject: (sessionIds: Array<DocumentSessionId>) =>
    __TAURI_INVOKE<null>("plugin:configs|close_project", { sessionIds }),
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
      sessionId: DocumentSessionId;
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
   * Everything wrong with one resolved root, anchored to the file and line a person has to open.
   *
   * Held with the resolution, so opening the panel a second time is a lookup rather than a second walk of every section
   * the root holds.
   */
  listFindings: (request: ConfigsResolvedRequest) =>
    __TAURI_INVOKE<Array<LtxAnchoredFinding>>("plugin:configs|list_findings", { request }),
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
   * Off the async worker even though a page is usually a map lookup: the root is normally resolved by the time anything
   * can ask for one, but "normally" is not a guarantee. A page asked for after the session was replaced would resolve a
   * whole include tree, and doing that on the IPC handler thread would stall every other command behind it.
   */
  readResolvedSections: (request: ConfigsReadSectionsRequest) =>
    __TAURI_INVOKE<Array<LtxResolvedSection>>("plugin:configs|read_resolved_sections", { request }),
  /**
   * What one section is judged by, and how it measures against that.
   *
   * `None` means the root does not hold the section, which a panel reaches by asking about a selection the index no
   * longer holds.
   */
  readSectionScheme: (request: ConfigsSectionRequest) =>
    __TAURI_INVOKE<{
      /** Engine identity of the entry point whose resolution this was read from. */
      entry: string;
      section: string;
      /** The `$scheme` the resolved section carries, absent when it carries none. */
      scheme: string | null;
      /** Whether a scheme file declares that name. False is itself a finding, and the verifier reports it as one. */
      isDeclared: boolean;
      /** Whether the declaration refuses fields it does not name and demands the ones it does not mark optional. */
      isStrict: boolean;
      /** The section the binding is written in, absent when this section writes it itself. */
      inheritedFrom: string | null;
      /** Every field the scheme declares and every field the section holds, merged. */
      fields: Array<LtxSchemeFieldReport>;
    } | null>("plugin:configs|read_section_scheme", { request }),
  /** Verifies LTX configs through the VFS, including archived files. */
  verifyDirectory: (request: ConfigsVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectVerifyResult>("plugin:configs|verify_directory", { request, jobId, progress }),
};
