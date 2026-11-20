use xrf_report::Report;
use xrf_spawn::SpawnFile;

/// A built root spawn and its report of what the build let through.
pub(crate) struct SpawnRootBuild {
  pub spawn: SpawnFile,
  pub report: Report,
}
