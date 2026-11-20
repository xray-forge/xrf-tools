use std::sync::{Arc, Mutex, PoisonError};

use crate::report::{SpriteListener, SpriteNotice};

/// Keeps every notice a run hands its listener, for a test asserting what a run noticed and in which order.
#[derive(Clone, Default)]
pub(crate) struct SpriteNoticeRecorder {
  notices: Arc<Mutex<Vec<SpriteNotice>>>,
}

impl SpriteNoticeRecorder {
  /// A listener that records into this recorder.
  pub(crate) fn new_listener(&self) -> Option<SpriteListener> {
    let notices: Arc<Mutex<Vec<SpriteNotice>>> = Arc::clone(&self.notices);

    Some(Arc::new(move |notice: &SpriteNotice| {
      notices
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(notice.clone());
    }))
  }

  /// Every notice recorded so far.
  pub(crate) fn list_notices(&self) -> Vec<SpriteNotice> {
    self.notices.lock().unwrap_or_else(PoisonError::into_inner).clone()
  }
}
