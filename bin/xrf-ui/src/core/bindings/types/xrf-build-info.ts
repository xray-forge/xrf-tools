// Auto-generated rust bindings. Do not edit it manually.

/**
 * Where a binary came from, as recorded when it was compiled.
 *
 * Every field is `Option` because a build script may not have run, or a value may be unavailable - a
 * build outside a Git checkout has no commit, and a local build has no workflow run. Reporting the
 * absence is more useful than substituting a plausible-looking default.
 */
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
export type BuildKind =
  /** Built on a developer machine. */
  | "local"
  /** Continuous integration artifact, built for turnaround rather than size. */
  | "development"
  /** Deliberate release build, carrying the full optimisation the release profile describes. */
  | "optimized";
