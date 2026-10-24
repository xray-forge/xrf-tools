// Auto-generated rust bindings. Do not edit it manually.

/** Where a binary came from, as recorded when it was compiled. */
export type BuildInfo = {
  /** Crate version of the binary itself. */
  version: string;
  /** How the binary was produced, which is what separates a fast nightly from a size-optimised release. */
  kind: BuildKind;
  /** Full commit the sources were at. */
  commit: string | null;
  /** Branch or tag the build ran from. */
  reference: string | null;
  /** Whether the checkout carried uncommitted changes, which only a local build can. */
  isDirty: boolean;
  /** RFC 3339 instant the build script ran. */
  builtAt: string | null;
  /** Target triple the binary runs on. */
  target: string | null;
  /** Compiler that produced it. */
  rustc: string | null;
  /** Cargo profile name, as opposed to the optimisation settings below. */
  profile: string | null;
  /** Optimisation level, link-time optimisation and codegen unit count, as cargo resolved them. */
  optimization: string | null;
  /** Identifier of the workflow run that produced the binary, absent for a local build. */
  runId: string | null;
};

/** Why a binary exists, which is the difference a downloaded artifact cannot show on its own. */
export enum EBuildKind {
  /** Built on a developer machine. */
  LOCAL = "local",
  /** Continuous integration artifact, built for turnaround rather than size. */
  DEVELOPMENT = "development",
  /** Release build, carrying the full optimisation the release profile describes. */
  OPTIMIZED = "optimized",
}

/** Every `EBuildKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type BuildKind = `${EBuildKind}`;
