// Auto-generated rust bindings. Do not edit it manually.

import { Channel } from "@tauri-apps/api/core";

import { invoke as __TAURI_INVOKE } from "#/invoke";
import { IIpcCommandSignature } from "#/signature";
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
} from "#/types/xrf-app-configs";
import { SessionId, SessionRestore, SessionSnapshot } from "#/types/xrf-app-core";
import { JobProgress } from "#/types/xrf-job";
import { LtxProjectFormatResult, LtxProjectVerifyResult } from "#/types/xrf-ltx";
import {
  LtxAnchoredFinding,
  LtxResolvedIndex,
  LtxResolvedSection,
  LtxSchemeFieldReport,
} from "#/types/xrf-ltx-inspect";

/** The wire name of every command of the `configs` plugin. */
export enum EConfigsCommand {
  CHECK_DIRECTORY_FORMAT = "plugin:configs|check_directory_format",
  CLOSE_PROJECT = "plugin:configs|close_project",
  FORMAT_DIRECTORY = "plugin:configs|format_directory",
  LIST_FINDINGS = "plugin:configs|list_findings",
  LIST_RESOLVED_SECTIONS = "plugin:configs|list_resolved_sections",
  OPEN_PROJECT = "plugin:configs|open_project",
  READ_DOCUMENT = "plugin:configs|read_document",
  READ_RESOLVED_SECTIONS = "plugin:configs|read_resolved_sections",
  READ_SECTION_SCHEME = "plugin:configs|read_section_scheme",
  RESTORE_PROJECT = "plugin:configs|restore_project",
  VERIFY_DIRECTORY = "plugin:configs|verify_directory",
}

/** Commands */
export const configsCommands = {
  /**
   * Report which LTX configs roots exposes are misformatted, in the format's own lane so a check never reads half a
   * rewrite.
   */
  checkDirectoryFormat: (request: ConfigsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectFormatResult>(EConfigsCommand.CHECK_DIRECTORY_FORMAT, { request, jobId, progress }),
  /** Close the configs projects `session_ids` name, committed or opening, and whatever each had resolved. */
  closeProject: (sessionIds: Array<SessionId>) => __TAURI_INVOKE<void>(EConfigsCommand.CLOSE_PROJECT, { sessionIds }),
  /** Rewrite the LTX configs roots exposes, as one commit, one format or format check at a time. */
  formatDirectory: (request: ConfigsFormatRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectFormatResult>(EConfigsCommand.FORMAT_DIRECTORY, { request, jobId, progress }),
  /**
   * Everything wrong with one resolved root, anchored to the file and line a person has to open; verified once and
   * answered as held.
   */
  listFindings: (request: ConfigsResolvedRequest) =>
    __TAURI_INVOKE<Array<LtxAnchoredFinding>>(EConfigsCommand.LIST_FINDINGS, { request }),
  /** Lists every section one entry point resolves to, named and counted. */
  listResolvedSections: (request: ConfigsResolvedRequest) =>
    __TAURI_INVOKE<LtxResolvedIndex>(EConfigsCommand.LIST_RESOLVED_SECTIONS, { request }),
  /** Open a configs project for browsing over the shared asset roots, read on the pool, and list what it holds. */
  openProject: (request: ConfigsOpenRequest) =>
    __TAURI_INVOKE<SessionSnapshot<ConfigsProjectDescriptor>>(EConfigsCommand.OPEN_PROJECT, { request }),
  /** Reads one config as the authored view renders it: its lines, and what only the parser knows about them. */
  readDocument: (request: ConfigsReadDocumentRequest) =>
    __TAURI_INVOKE<ConfigsDocument>(EConfigsCommand.READ_DOCUMENT, { request }),
  /** Reads the bodies of the named sections of one entry point. */
  readResolvedSections: (request: ConfigsReadSectionsRequest) =>
    __TAURI_INVOKE<Array<LtxResolvedSection>>(EConfigsCommand.READ_RESOLVED_SECTIONS, { request }),
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
    } | null>(EConfigsCommand.READ_SECTION_SCHEME, { request }),
  /**
   * The configs project `tool` holds in the asking window, the newest it opened, for a page attaching to it after a
   * reload.
   */
  restoreProject: (tool: string) =>
    __TAURI_INVOKE<SessionRestore<ConfigsProjectDescriptor>>(EConfigsCommand.RESTORE_PROJECT, { tool }),
  /** Verifies LTX configs through the VFS, including archived files, one verification at a time. */
  verifyDirectory: (request: ConfigsVerifyRequest, jobId: string, progress: Channel<JobProgress>) =>
    __TAURI_INVOKE<LtxProjectVerifyResult>(EConfigsCommand.VERIFY_DIRECTORY, { request, jobId, progress }),
};

/** What each command of the `configs` plugin takes and resolves to, keyed by its wire name. */
export interface IConfigsCommandSignatures {
  [EConfigsCommand.CHECK_DIRECTORY_FORMAT]: IIpcCommandSignature<
    typeof configsCommands.checkDirectoryFormat,
    ["request", "jobId", "progress"]
  >;
  [EConfigsCommand.CLOSE_PROJECT]: IIpcCommandSignature<typeof configsCommands.closeProject, ["sessionIds"]>;
  [EConfigsCommand.FORMAT_DIRECTORY]: IIpcCommandSignature<
    typeof configsCommands.formatDirectory,
    ["request", "jobId", "progress"]
  >;
  [EConfigsCommand.LIST_FINDINGS]: IIpcCommandSignature<typeof configsCommands.listFindings, ["request"]>;
  [EConfigsCommand.LIST_RESOLVED_SECTIONS]: IIpcCommandSignature<
    typeof configsCommands.listResolvedSections,
    ["request"]
  >;
  [EConfigsCommand.OPEN_PROJECT]: IIpcCommandSignature<typeof configsCommands.openProject, ["request"]>;
  [EConfigsCommand.READ_DOCUMENT]: IIpcCommandSignature<typeof configsCommands.readDocument, ["request"]>;
  [EConfigsCommand.READ_RESOLVED_SECTIONS]: IIpcCommandSignature<
    typeof configsCommands.readResolvedSections,
    ["request"]
  >;
  [EConfigsCommand.READ_SECTION_SCHEME]: IIpcCommandSignature<typeof configsCommands.readSectionScheme, ["request"]>;
  [EConfigsCommand.RESTORE_PROJECT]: IIpcCommandSignature<typeof configsCommands.restoreProject, ["tool"]>;
  [EConfigsCommand.VERIFY_DIRECTORY]: IIpcCommandSignature<
    typeof configsCommands.verifyDirectory,
    ["request", "jobId", "progress"]
  >;
}
