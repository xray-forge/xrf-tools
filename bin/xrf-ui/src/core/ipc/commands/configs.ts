// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "@/core/ipc/invoke";
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
  SessionId,
} from "@/core/ipc/types/xrf-app";
import { JobProgress } from "@/core/ipc/types/xrf-job";
import { LtxProjectFormatResult, LtxProjectVerifyResult } from "@/core/ipc/types/xrf-ltx";
import {
  LtxAnchoredFinding,
  LtxInventory,
  LtxResolvedIndex,
  LtxResolvedSection,
  LtxSchemeFieldReport,
} from "@/core/ipc/types/xrf-ltx-inspect";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Commands */
export const configsCommands = {
  /** Report which LTX configs roots exposes are misformatted. */
  checkDirectoryFormat: (request: ConfigsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|check_directory_format", { request, jobId, progress }),
  /** Closes the configs project, releasing its mounts and whatever it had resolved. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<null>("plugin:configs|close_project", { sessionIds }),
  /** Rewrite the LTX configs roots exposes. */
  formatDirectory: (request: ConfigsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectFormatResult>("plugin:configs|format_directory", { request, jobId, progress }),
  /** What configs project is open, for a frontend restoring itself after a reload. */
  getProject: () =>
    __TAURI_INVOKE<{
      /** Identity every later read is addressed by. */
      sessionId: SessionId;
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
  /** Everything wrong with one resolved root, anchored to the file and line a person has to open. */
  listFindings: (request: ConfigsResolvedRequest) =>
    __TAURI_INVOKE<Array<LtxAnchoredFinding>>("plugin:configs|list_findings", { request }),
  /** Lists every section one entry point resolves to, named and counted. */
  listResolvedSections: (request: ConfigsResolvedRequest) =>
    __TAURI_INVOKE<LtxResolvedIndex>("plugin:configs|list_resolved_sections", { request }),
  /** Opens a configs project for browsing, and lists what it holds. */
  openProject: (request: ConfigsOpenRequest) =>
    __TAURI_INVOKE<ConfigsProjectDescriptor>("plugin:configs|open_project", { request }),
  /** Reads one config as the authored view renders it: its lines, and what only the parser knows about them. */
  readDocument: (request: ConfigsReadDocumentRequest) =>
    __TAURI_INVOKE<ConfigsDocument>("plugin:configs|read_document", { request }),
  /** Reads the bodies of the named sections of one entry point. */
  readResolvedSections: (request: ConfigsReadSectionsRequest) =>
    __TAURI_INVOKE<Array<LtxResolvedSection>>("plugin:configs|read_resolved_sections", { request }),
  /** What one section is judged by, and how it measures against that. */
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
