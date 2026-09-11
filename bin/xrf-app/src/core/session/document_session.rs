use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::session::{DocumentSessionId, DocumentSnapshot};
use crate::core::types::TauriResult;

/// Publishes complete documents atomically, retaining the last success while a replacement is pending.
pub(crate) struct DocumentSession<T> {
  name: &'static str,
  state: Arc<Mutex<SessionState<T>>>,
}

impl<T> Clone for DocumentSession<T> {
  fn clone(&self) -> Self {
    Self {
      name: self.name,
      state: Arc::clone(&self.state),
    }
  }
}

struct SessionState<T> {
  opening: Option<DocumentSessionId>,
  opened: Option<Arc<DocumentSnapshot<T>>>,
}

impl<T> DocumentSession<T> {
  pub fn new(name: &'static str) -> Self {
    Self {
      name,
      state: Arc::new(Mutex::new(SessionState {
        opening: None,
        opened: None,
      })),
    }
  }

  /// Reserves publication before doing work, without invalidating the committed document's readers.
  pub fn begin_open(&self, id: DocumentSessionId) -> TauriResult<()> {
    self.lock()?.opening = Some(id);

    Ok(())
  }

  /// Reserves a reload only while the document it reloads is still committed.
  pub fn begin_reload(
    &self,
    id: DocumentSessionId,
    previous: DocumentSessionId,
  ) -> TauriResult<Arc<DocumentSnapshot<T>>> {
    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    let opened: Arc<DocumentSnapshot<T>> = state
      .opened
      .as_ref()
      .filter(|opened| opened.session_id == previous)
      .cloned()
      .ok_or_else(|| format!("The {} session has changed or is closed", self.name))?;

    state.opening = Some(id);

    Ok(opened)
  }

  /// Publishes only the newest opening; parsing and resource acquisition happen before this call.
  pub fn commit_open(&self, id: DocumentSessionId, document: T) -> TauriResult<Arc<DocumentSnapshot<T>>> {
    let opened: Arc<DocumentSnapshot<T>> = Arc::new(DocumentSnapshot {
      session_id: id,
      document,
    });

    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    if state.opening != Some(id) {
      return Err(format!(
        "The {} opening was superseded by another open or close",
        self.name
      ));
    }

    let previous: Option<Arc<DocumentSnapshot<T>>> = state.opened.replace(Arc::clone(&opened));

    state.opening = None;

    drop(state);
    drop(previous);

    Ok(opened)
  }

  /// Restores the committed document without copying its content or exposing the session lock.
  pub fn get(&self) -> TauriResult<Option<Arc<DocumentSnapshot<T>>>> {
    Ok(self.lock()?.opened.clone())
  }

  /// Takes the exact document requested, never whichever document happens to be open now.
  pub fn require(&self, id: DocumentSessionId) -> TauriResult<Arc<DocumentSnapshot<T>>> {
    self
      .get()?
      .filter(|opened| opened.session_id == id)
      .ok_or_else(|| format!("The {} session has changed or is closed", self.name))
  }

  /// Replaces a saved snapshot only if no other publication has replaced the snapshot it was based on.
  pub fn replace(
    &self,
    previous: &Arc<DocumentSnapshot<T>>,
    document: T,
  ) -> TauriResult<Option<Arc<DocumentSnapshot<T>>>> {
    let opened: Arc<DocumentSnapshot<T>> = Arc::new(DocumentSnapshot {
      session_id: previous.session_id,
      document,
    });

    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    if !state
      .opened
      .as_ref()
      .is_some_and(|current| Arc::ptr_eq(current, previous))
    {
      return Ok(None);
    }

    let discarded: Option<Arc<DocumentSnapshot<T>>> = state.opened.replace(Arc::clone(&opened));

    drop(state);
    drop(discarded);

    Ok(Some(opened))
  }

  /// Closes only the caller's committed or pending openings. A delayed teardown cannot close its successor.
  pub fn close(&self, ids: &[DocumentSessionId]) -> TauriResult<()> {
    drop(self.detach(ids)?);

    Ok(())
  }

  /// Detaches ownership so a caller can dispose a large document on its execution pool.
  pub fn detach(&self, ids: &[DocumentSessionId]) -> TauriResult<Option<Arc<DocumentSnapshot<T>>>> {
    let mut state: MutexGuard<SessionState<T>> = self.lock()?;

    if state.opening.is_some_and(|id| ids.contains(&id)) {
      state.opening = None;
    }

    let previous: Option<Arc<DocumentSnapshot<T>>> = if state
      .opened
      .as_ref()
      .is_some_and(|opened| ids.contains(&opened.session_id))
    {
      state.opened.take()
    } else {
      None
    };

    drop(state);

    Ok(previous)
  }

  fn lock(&self) -> TauriResult<MutexGuard<'_, SessionState<T>>> {
    self
      .state
      .lock()
      .map_err(|error| format!("The {} session is unavailable: {error}", self.name))
  }
}
