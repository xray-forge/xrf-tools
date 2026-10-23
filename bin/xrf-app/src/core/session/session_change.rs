use serde::Serialize;
use xrf_vfs::XrayRevision;

use crate::core::session::{SessionId, SessionRootChange};

/// What one committed change did to one open session, as a change event reports it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionChange {
  pub session_id: SessionId,
  /// The revision the session's snapshot is at now: the change's where it moved, its own where it could not.
  pub revision: XrayRevision,
  pub root: SessionRootChange,
  /// The parts the session let go, by name and sorted: built from something the change moved, or from the value it
  /// replaced. A subscriber refetches what it showed of them.
  pub stale_parts: Vec<&'static str>,
}
