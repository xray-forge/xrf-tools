use std::sync::{Arc, Weak};

use crate::session::{SessionId, SessionSnapshot};

/// What a session holds under its lock: the opening that may publish, the committed snapshot, and what a commit's
/// follow is doing with it.
pub(crate) struct SessionState<T> {
  pub(crate) opening: Option<SessionId>,
  pub(crate) opened: Option<Arc<SessionSnapshot<T>>>,
  /// The snapshot a deferred derivation is reading again, so a commit landing meanwhile starts no second one.
  pub(crate) deriving: Option<Arc<SessionSnapshot<T>>>,
  /// The snapshot reported stale, which no later revision makes current, so it is reported once rather than on every
  /// commit until something replaces it; held weakly, so a replaced one is let go.
  pub(crate) stale: Weak<SessionSnapshot<T>>,
}
