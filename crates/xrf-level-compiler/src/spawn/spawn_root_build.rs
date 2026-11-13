use xrf_report::Report;
use xrf_spawn::SpawnFile;

/// A root spawn as the build gives it, and its report: the level connections it left out and the restrictors cutting
/// an AI map apart, each a finding.
pub struct SpawnRootBuild {
  pub spawn: SpawnFile,
  pub report: Report,
}
