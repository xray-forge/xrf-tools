use std::collections::HashMap;
use std::sync::Arc;

use crate::core::session::{SessionId, SessionRelease, SessionScope};

/// What `SessionScopes` holds behind its lock: the scope each opening is bound to, and every session holding openings.
#[derive(Default)]
pub(super) struct SessionScopesState {
  pub bindings: HashMap<SessionId, SessionScope>,
  pub sessions: Vec<Arc<dyn SessionRelease>>,
}
