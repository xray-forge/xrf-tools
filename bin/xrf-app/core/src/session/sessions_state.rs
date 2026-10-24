use std::collections::HashMap;

use crate::session::{Session, SessionId};

/// What `Sessions` holds behind its lock, shared by every clone: the template a new handle starts as, the handles, and
/// when each last committed an open.
pub(super) struct SessionsState<T> {
  /// What a new handle starts as: no opening, and the derivation every handle reads its value again with.
  pub template: Session<T>,
  pub handles: HashMap<SessionId, Session<T>>,
  /// When each handle last committed an open, in order, so a restore answers the newest.
  pub committed: HashMap<SessionId, u64>,
  pub sequence: u64,
}

impl<T> SessionsState<T> {
  pub fn new(template: Session<T>) -> Self {
    Self {
      template,
      handles: HashMap::new(),
      committed: HashMap::new(),
      sequence: 0,
    }
  }
}
