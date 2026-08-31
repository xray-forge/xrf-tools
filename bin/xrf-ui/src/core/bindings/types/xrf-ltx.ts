// Auto-generated rust bindings. Do not edit it manually.

import { XrfError } from "@/core/bindings/types/xrf-error";
import { JobOutcome } from "@/core/bindings/types/xrf-job";

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
  errors: Array<XrfError>;
  invalidSections: number;
  skippedSections: number;
  totalFiles: number;
  totalSections: number;
  validSections: number;
};
