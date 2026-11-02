// Auto-generated rust bindings. Do not edit it manually.

export type CheckId = string;

/** Immutable report data for one command check. */
export type CheckReport = {
  /**
   * `null` when the check did not measure itself; a millisecond count otherwise, like every other
   * duration on the wire. Never omitted, because the IPC bindings cannot type a conditionally skipped
   * field.
   */
  duration: number | null;
  findings: Array<Finding>;
  id: CheckId;
  status: Status;
};

/** A single rule violation or checker error. */
export type Finding = {
  message: string;
  ruleId: RuleId;
  subject: string | null;
};

/** Immutable, finalized account of a command's checks and findings. */
export type Report = {
  checks: Array<CheckReport>;
  status: Status;
};

export type RuleId = string;

/** The outcome of a completed check. */
export enum EStatus {
  ERROR = "error",
  FAILED = "failed",
  INCOMPLETE = "incomplete",
  PASSED = "passed",
  SKIPPED = "skipped",
}

/** Every `EStatus` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type Status = `${EStatus}`;
