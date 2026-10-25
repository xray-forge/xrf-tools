use serde::Serialize;
use xrf_report::Report;
use xrf_vfs::{XrayPathCollision, XrayShadowingEntry};

use crate::commands::archive::verify::archive_verify_override_report::ArchiveVerifyOverrideReport;

/// The verdict `archive verify` reached, with what the volume set's fold found beside it.
///
/// Deposited whether the verdict passed or failed, because a failing check is exactly when the findings explaining it
/// are worth reporting.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVerifyReport {
  checked: usize,
  /// Engine paths the set answers with more than one copy, reported beside the verdict without joining it.
  overrides: Vec<ArchiveVerifyOverrideReport>,
  /// Entries the volume set holds that no engine lookup can reach, reported beside the verdict without joining it.
  ///
  /// This command judges payloads: whether the bytes an entry names read back. Whether the engine can *address* that
  /// entry is a different question, and one a mount answers, so it is reported without joining the status —
  /// `gamedata verify` is where a project's reachability belongs in a verdict. Reporting it at all keeps a clean CRC
  /// sweep from implying a volume set nothing is wrong with.
  unreachable: Vec<XrayPathCollision>,
  #[serde(flatten)]
  report: Report,
}

impl ArchiveVerifyReport {
  /// The payloads `report` judged out of `checked` entries, beside the set's overrides and unreachable entries, which
  /// are not defects: a set full of working patches still passes.
  pub fn new(
    checked: usize,
    overrides: &[XrayShadowingEntry],
    unreachable: Vec<XrayPathCollision>,
    report: Report,
  ) -> Self {
    Self {
      checked,
      overrides: overrides.iter().map(ArchiveVerifyOverrideReport::from).collect(),
      unreachable,
      report,
    }
  }
}
