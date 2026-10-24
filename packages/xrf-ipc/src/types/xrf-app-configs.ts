// Auto-generated rust bindings. Do not edit it manually.

import { SessionId } from "#/types/xrf-app-core";
import { LtxAnchoredFinding, LtxFileStructure, LtxFileText, LtxInventory } from "#/types/xrf-ltx-inspect";
import { XrayRoots } from "#/types/xrf-vfs";

/** Checking that a directory's configs are formatted, writing nothing. */
export const CONFIGS_JOB_CHECK_FORMAT = "configs.check-format" as const;

/** Formatting a directory's configs as one commit. */
export const CONFIGS_JOB_FORMAT = "configs.format" as const;

/** Verifying a directory's configs. */
export const CONFIGS_JOB_VERIFY = "configs.verify" as const;

/** One config as the authored view renders it. */
export type ConfigsDocument = {
  text: LtxFileText;
  structure: LtxFileStructure;
  /** What is wrong with this file itself: it will not parse, or an `#include` reached nothing. */
  findings: Array<LtxAnchoredFinding>;
};

/** What a config formatting run, or a check of one, was asked to do. */
export type ConfigsFormatRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
};

/** What opening a configs project for browsing was asked to do. */
export type ConfigsOpenRequest = {
  sessionId: SessionId;
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Whether to resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/** What one open of the configs explorer answers with, under the opening's identity. */
export type ConfigsProjectDescriptor = {
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
};

/** Which config of which open a reader wants. */
export type ConfigsReadDocumentRequest = {
  /** The open this read is addressed to; a read naming one closed is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the config to read. */
  path: string;
};

/** Which sections of a resolved root a page wants. */
export type ConfigsReadSectionsRequest = {
  /** The open this read is addressed to; a read naming one closed is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point the sections belong to. */
  entry: string;
  /** Sections to read, as the index named them. */
  names: Array<string>;
};

/** Which resolved root a reader wants, of which open. */
export type ConfigsResolvedRequest = {
  /** The open this read is addressed to; a read naming one closed is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point to resolve. */
  entry: string;
};

/** Which section of a resolved root a reader wants explained. */
export type ConfigsSectionRequest = {
  /** The open this read is addressed to; a read naming one closed is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point the section belongs to. */
  entry: string;
  /** The section to explain, as the index named it. */
  section: string;
};

/** What a config verification was asked to do. */
export type ConfigsVerifyRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Whether to resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
  /**
   * Whether to check what only a case-sensitive file system breaks on too, such as an include naming a file in
   * another case, which the vanilla game data does.
   */
  isStrict: boolean;
};
