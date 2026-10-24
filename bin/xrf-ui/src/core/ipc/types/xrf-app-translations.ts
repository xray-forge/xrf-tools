// Auto-generated rust bindings. Do not edit it manually.

import { SessionId, SessionSnapshot } from "@/core/ipc/types/xrf-app-core";
import {
  TranslationLanguage,
  TranslationProjectDescriptor,
  TranslationProjectMode,
} from "@/core/ipc/types/xrf-translation";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";

/** Compiling translation sources into string tables. */
export const TRANSLATIONS_JOB_BUILD = "translations.build" as const;

/** Checking that translation sources are normalized, writing nothing. */
export const TRANSLATIONS_JOB_CHECK_FORMAT = "translations.check-format" as const;

/** Normalizing translation sources. */
export const TRANSLATIONS_JOB_FORMAT = "translations.format" as const;

/** Importing raw XML string tables into translation sources. */
export const TRANSLATIONS_JOB_PARSE = "translations.parse" as const;

/** Reporting which translations are missing from which languages. */
export const TRANSLATIONS_JOB_VERIFY = "translations.verify" as const;

/** Every `kind` the `TranslationSaveOutcome` union is told apart by, so a switch or a comparison names one. */
export enum ETranslationSaveOutcome {
  /** The edits are on disk, and this is the project as it now reads. */
  SAVED = "saved",
  /** The edits are on disk, but the tool that asked for them closed the project while they were being written. */
  STALE = "stale",
}

/** How a save ended, once its edits were on disk. */
export type TranslationSaveOutcome =
  /** The edits are on disk, and this is the project as it now reads. */
  | { kind: "saved"; project: SessionSnapshot<TranslationProjectDescriptor> }
  /** The edits are on disk, but the tool that asked for them closed the project while they were being written. */
  | { kind: "stale" };

/** What a build was asked to do. */
export type TranslationsBuildRequest = {
  /** Where the sources are read from, through the VFS. */
  roots: XrayRoots;
  /** Where inside those roots to look, or nothing for the whole set. */
  prefix: string | null;
  /** The language to build, or `all`. */
  language: TranslationLanguage;
  /** Directory the string tables are written into, which is always a host path. */
  outputDir: string;
  /** Whether to sort entries within each table. */
  isSorted: boolean;
};

/** What a translation formatting run, or a check of one, was asked to do. */
export type TranslationsFormatRequest = {
  /** Project directory to format. */
  directory: string;
  /** Line endings to write, or nothing to keep what each file already uses. */
  lineEndings: string | null;
};

/** Inputs and caller-owned identity of one translations opening. */
export type TranslationsOpenRequest = {
  sessionId: SessionId;
  roots: XrayRoots;
  mode: TranslationProjectMode;
  prefix: string | null;
};

/** What an import was asked to do. */
export type TranslationsParseRequest = {
  /** Roots holding the raw XML, read through the VFS so an installation imports like a loose tree. */
  roots: XrayRoots;
  /** The language every entry this run reads is filed under. Never `all`. */
  language: TranslationLanguage;
  /** Where inside those roots to look, or nothing to let the run resolve it. */
  prefix: string | null;
  /** Directory the JSON sources are written to, which may already hold some. */
  outputDir: string;
  /** Restrict the run to one table, by the file name it has in the scope. */
  file: string | null;
  /** Let incoming text replace existing text that differs, instead of keeping what is there. */
  isOverwrite: boolean;
  /** Do everything except write, so a caller can see what a run would change. */
  isDryRun: boolean;
};

/** What a translation verification was asked to do. */
export type TranslationsVerifyRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Language the check is about, or `all`. */
  language: TranslationLanguage;
};
