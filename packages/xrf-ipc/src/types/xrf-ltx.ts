// Auto-generated rust bindings. Do not edit it manually.

import { JobOutcome } from "#/types/xrf-job";

/**
 * One thing wrong with a config tree, placed at the entry point it was found under and the config it is about.
 *
 * Paths are engine identities. Where a person needs the file on disk rather than the logical path, the surface that
 * shows the finding renders it, through [`crate::LtxProject::with_reported_paths`].
 */
export type LtxFinding = {
  kind: LtxFindingKind;
  /**
   * The entry point whose resolution the finding was produced under.
   *
   * Kept beside `file` rather than instead of it: the entry point is the unit a caller re-runs and the only thing
   * that explains why two files were read together, but it is not where the section is written.
   */
  entry: string;
  /**
   * The config to open, where one is known.
   *
   * `None` means nothing recorded which config declares the section, which a patch dialect answers for a section
   * created by an override of something nothing declares.
   */
  file: string | null;
  section: string | null;
  /** The field it is about, or `None` for a finding about the section as a whole. */
  field: string | null;
  message: string;
  /** What the engine does with the same input, where the dialect said so. */
  engineBehaviour: string | null;
};

/** What kind of thing went wrong. */
export enum ELtxFindingKind {
  /** The file would not parse. Carries its own line, from the parser. */
  PARSE = "parse",
  /** A section broke the scheme it is bound to, or is bound to one nothing declares. */
  SCHEME = "scheme",
  /** The dialect had something to say about the root: a statement the engine loads and then silently ignores. */
  DIALECT = "dialect",
  /** An `#include` reached no file. */
  INCLUDE = "include",
  /**
   * A config declares keys above its first section header, where no engine loads them. Not raised for a config
   * that declares no section at all: that is a list, read as data rather than loaded as an ini.
   */
  ROOT_KEYS = "rootKeys",
  /** An entry point that could not be resolved, so nothing reached through it was judged. */
  UNREADABLE = "unreadable",
}

/** Every `ELtxFindingKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LtxFindingKind = `${ELtxFindingKind}`;

export type LtxProjectFormatResult = {
  /**
   * Whether the run reached the end of the set or was stopped between files.
   *
   * A stopped rewrite leaves the files it had already formatted formatted and the rest untouched, which is a state
   * running it again resolves. Nothing is removed and nothing is half-written.
   */
  outcome: JobOutcome;
  /** Everything the run took, measured from when its caller created the job handle. */
  duration: number;
  /**
   * How much of `duration` had already passed when the per-file work began.
   *
   * Mounting the roots, indexing the virtual filesystem, assembling the project and resolving its includes all happen
   * before a single file is read, and on a cold filesystem they dominate: a run reporting only its own loop told the
   * user one second where they had waited fifteen. Named rather than folded away, so the split stays readable.
   */
  startupDuration: number;
  invalidFiles: number;
  toFormat: Array<string>;
  totalFiles: number;
  validFiles: number;
};

export type LtxProjectVerifyResult = {
  /**
   * Whether the run reached the end of the project or was stopped between files.
   *
   * A stopped run reports the findings it had reached, so this is what separates "these are the problems" from
   * "these are the problems found so far" - the one way a partial check can mislead.
   */
  outcome: JobOutcome;
  checkedFields: number;
  checkedSections: number;
  /** Everything the run took, measured from when its caller created the job handle. */
  duration: number;
  /**
   * How much of `duration` had already passed when the per-file work began.
   *
   * Mounting the roots, indexing the virtual filesystem, assembling the project and resolving its includes all happen
   * before a single file is read, and on a cold filesystem they dominate: a run reporting only its own loop told the
   * user one second where they had waited fifteen. Named rather than folded away, so the split stays readable.
   */
  startupDuration: number;
  /** Everything wrong with the project, every one of which fails the verification. */
  findings: Array<LtxFinding>;
  invalidSections: number;
  skippedSections: number;
  totalFiles: number;
  totalSections: number;
  validSections: number;
};
