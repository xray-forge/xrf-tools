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

/** A single rule violation or checker error, and whether it fails its check. */
export type Finding = {
  message: string;
  ruleId: RuleId;
  severity: Severity;
  subject: string | null;
};

/** Immutable, finalized account of a command's checks and findings. */
export type Report = {
  checks: Array<CheckReport>;
  status: Status;
};

export type RuleId = string;

/**
 * Whether a finding fails its check, or is reported and fails it only when the run is strict.
 *
 * The producer that knows what the defect costs decides, once, at the finding: a reader of a report never has to know
 * a check's rules to tell what failed it.
 */
export enum ESeverity {
  /** What the consumer cannot work with: the engine asserts on it or refuses it, or the file does not read. */
  FAILURE = "failure",
  /** What the consumer tolerates: it works around it, shows something less, or never reaches it. */
  WARNING = "warning",
}

/** Every `ESeverity` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type Severity = `${ESeverity}`;

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
