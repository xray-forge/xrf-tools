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
  duration: number;
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
  duration: number;
  errors: Array<XrfError>;
  invalidSections: number;
  skippedSections: number;
  totalFiles: number;
  totalSections: number;
  validSections: number;
};
