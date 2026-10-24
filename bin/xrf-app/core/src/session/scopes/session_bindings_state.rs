use std::collections::HashMap;
use std::sync::Arc;

use rayon::ThreadPool;

use crate::session::{SessionId, SessionScope};

/// What `SessionBindings` holds behind its lock: the scope each opening is bound to, and the pool what a closed one held
/// is dropped on.
#[derive(Default)]
pub(super) struct SessionBindingsState {
  pub bindings: HashMap<SessionId, SessionScope>,
  /// Where a closed document is dropped, none dropping it where it closes.
  pub pool: Option<Arc<ThreadPool>>,
}
