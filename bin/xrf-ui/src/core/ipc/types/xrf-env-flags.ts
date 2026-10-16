// Auto-generated rust bindings. Do not edit it manually.

/** Who sets a variable, and when it is read. */
export enum EEnvFlagKind {
  /** Read by a running binary, set by whoever starts it. */
  RUNTIME = "runtime",
  /** Read by a third-party library; the application only reports it. */
  PASS_THROUGH = "passThrough",
  /** Read while the workspace builds, by a build script, `cargo make` or CI. */
  BUILD = "build",
  /** Set by a build script for the compiler, never by hand. */
  INTERNAL = "internal",
}

/** Every `EEnvFlagKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type EnvFlagKind = `${EEnvFlagKind}`;

/** A variable's state in a running binary: what it does, and its value if set. */
export type EnvFlagReading = {
  name: string;
  kind: EnvFlagKind;
  /** What it does, in one sentence. */
  description: string;
  /** What applies while it is unset or set to an invalid value. */
  default: string;
  /** Its value as set, or none if it is unset. */
  value: string | null;
  /** The value that applies, in the table's spelling; none if it is unset or invalid, so the default applies. */
  applied: string | null;
};
