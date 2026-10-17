use serde::Serialize;
use xrf_report::Status;
use xrf_vfs::{XrayPathCollision, XrayShadowingEntry};

use crate::commands::archive::verify::archive_verify_finding_report::ArchiveVerifyFindingReport;
use crate::commands::archive::verify::archive_verify_override_report::ArchiveVerifyOverrideReport;

/// The verdict `archive verify` reached.
///
/// Deposited whether the verdict passed or failed, because a failing check is exactly when the
/// findings explaining it are worth reporting. The status reuses [`Status`] so a consumer reads one
/// vocabulary for check outcomes across the CLI.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVerifyReport {
  checked: usize,
  /// Entries the volume set holds that no engine lookup can reach, reported beside the verdict without joining it.
  ///
  /// This command judges payloads: whether the bytes an entry names read back. Whether the engine can *address* that
  /// entry is a different question, and one a mount answers, so it is reported without joining the status —
  /// `gamedata verify` is where a project's reachability belongs in a verdict. Reporting it at all keeps a clean CRC
  /// sweep from implying a volume set nothing is wrong with.
  collisions: Vec<XrayPathCollision>,
  /// Engine paths the set answers with more than one copy, reported beside the verdict without joining it.
  overrides: Vec<ArchiveVerifyOverrideReport>,
  findings: Vec<ArchiveVerifyFindingReport>,
  status: Status,
}

impl ArchiveVerifyReport {
  pub fn new(
    checked: usize,
    overrides: &[XrayShadowingEntry],
    collisions: Vec<XrayPathCollision>,
    findings: Vec<ArchiveVerifyFindingReport>,
  ) -> Self {
    Self {
      // Overrides are deliberately absent from this: they are not defects, and a set full of working patches must
      // still pass.
      status: Status::from_is_valid(findings.is_empty()),
      checked,
      collisions,
      overrides: overrides.iter().map(ArchiveVerifyOverrideReport::from).collect(),
      findings,
    }
  }
}
