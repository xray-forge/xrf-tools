use serde::Serialize;
use xrf_vfs::XrayPathCollision;

use crate::plugins::archives::browse::archive_world_entry::ArchiveWorldEntry;

/// What one fold of a subject onto engine identities found: the copies a patch buried, and the copies nothing reaches.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOverrideReport {
  /// Engine paths this subject answers with more than one copy, winner first, ordered by path.
  pub overridden: Vec<ArchiveWorldEntry>,
  /// Copies no lookup reaches, because another entry of the same source already claims their engine path.
  pub unreachable: Vec<XrayPathCollision>,
}
