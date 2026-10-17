use serde::Serialize;
use xrf_vfs::XraySkippedMount;

/// One declared source that could not be opened, so nothing the run reports covers it.
///
/// Shared because the fact belongs to a set of mounts rather than to any one command: `gamedata list` answers for what
/// it enumerated and `gamedata verify` for what it judged, and the same omission reported in two shapes is the
/// per-command drift the reporting contract exists to remove. Each command names the field, since `skipped` reads
/// unambiguously in a listing and collides with a skipped check verdict in a verification.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedMountReport {
  origin: String,
  path: String,
  reason: String,
}

impl SkippedMountReport {
  pub fn new(skipped: &XraySkippedMount) -> Self {
    Self {
      origin: skipped.origin.clone(),
      path: xrf_utils::to_portable_path_string(&skipped.path),
      reason: skipped.reason.clone(),
    }
  }

  pub fn list(skipped: &[XraySkippedMount]) -> Vec<Self> {
    skipped.iter().map(Self::new).collect()
  }
}
