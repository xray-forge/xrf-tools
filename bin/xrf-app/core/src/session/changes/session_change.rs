use serde::Serialize;
use xrf_vfs::XrayRevision;

use crate::session::{SessionId, SessionRootChange};

/// What one committed change did to one open session, as a change event reports it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionChange {
  pub session_id: SessionId,
  /// The revision the session's snapshot is at now: the change's where it moved, its own where it could not.
  pub revision: XrayRevision,
  pub root: SessionRootChange,
  /// The parts the session let go, by name and sorted: built from something the change moved, or from the value it
  /// replaced. A subscriber refetches what it showed of them.
  pub stale_parts: Vec<&'static str>,
  /// Whether a view drawn from the session is built anew rather than updated: its value was derived again, or, the value
  /// kept, a part its owner builds the view from went stale (`Session::with_view_parts`).
  pub is_view_renewed: bool,
}
