// Auto-generated rust bindings. Do not edit it manually.

/** Who sets a variable, and when it is read. */
export enum EEnvFlagKind {
  /** Read by a running binary, set by whoever starts it. */
  RUNTIME = "runtime",
  /** Another library's, which it reads itself and the application reports. */
  PASS_THROUGH = "passThrough",
  /** Read while the workspace builds, by a build script, `cargo make` or CI. */
  BUILD = "build",
  /** Set by a build script for the compiler, never by hand. */
  INTERNAL = "internal",
}

/** Every `EEnvFlagKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type EnvFlagKind = `${EEnvFlagKind}`;

/** A variable as a running binary found it: what it is for and what it was set to, if anything. */
export type EnvFlagReading = {
  name: string;
  kind: EnvFlagKind;
  /** What it does, one sentence. */
  description: string;
  /** What applies while it is unset or set to what it does not take. */
  default: string;
  /** What it was set to, none where unset. */
  value: string | null;
  /** The value it took, in its own spelling: none where unset, or set to what it does not take, so the default applies. */
  applied: string | null;
};
