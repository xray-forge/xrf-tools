use serde::Serialize;
use xrf_vfs::XrayRevision;

use crate::core::session::SessionId;

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

/// What became of a session's value, the root every part derives from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub(crate) enum SessionRootChange {
  /// Nothing it was read from moved: the next snapshot shares it.
  Kept,
  /// Something it was read from moved, and its plugin derived it again.
  Derived,
  /// Something it was read from moved, and its plugin derives it again off the commit: the session shows it as it was
  /// until a later event reports it `Derived` or `Failed`.
  Deriving,
  /// Something it was read from moved, and nothing derives it again: the session shows it as it was.
  Stale,
  /// Deriving it again failed: the session stays at the snapshot it had.
  Failed { reason: String },
}
