use serde::Serialize;
use xrf_report::Status;
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayAssetContainer, XrayPathCollision, XrayShadowingEntry};

/// One engine path the set answers with more than one copy, as a report states it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVerifyOverrideReport {
  logical_path: String,
  /// Where the copy a lookup reaches sits.
  container: String,
  /// The copies behind it, in precedence order.
  hidden: Vec<String>,
}

impl From<&XrayShadowingEntry> for ArchiveVerifyOverrideReport {
  fn from(entry: &XrayShadowingEntry) -> Self {
    Self {
      container: to_portable_container(entry.get_asset().get_container()),
      hidden: entry
        .shadowed
        .iter()
        .map(|copy| to_portable_container(copy.asset.get_container()))
        .collect(),
      logical_path: entry.get_logical_path().to_string(),
    }
  }
}

/// Where a container sits, spelled the one way every report spells a path.
fn to_portable_container(container: &XrayAssetContainer) -> String {
  match container {
    XrayAssetContainer::Directory { root, relative_path } => to_portable_path_string(root.join(relative_path)),
    XrayAssetContainer::Archive { path } => to_portable_path_string(path),
  }
}

/// One payload that could not be read back.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveVerifyFindingReport {
  message: String,
  name: String,
}

impl ArchiveVerifyFindingReport {
  pub fn new(name: &str, message: String) -> Self {
    Self {
      message,
      name: String::from(name),
    }
  }
}

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
