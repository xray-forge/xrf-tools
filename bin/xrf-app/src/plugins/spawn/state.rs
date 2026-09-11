use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use xrf_db::SpawnFile;

use crate::core::types::TauriResult;
use crate::plugins::spawn::{SpawnSessionDescriptor, SpawnSessionId};

/// Owns the committed file separately from the newest opening allowed to replace it.
#[derive(Clone)]
pub struct SpawnFileState {
  session: Arc<Mutex<SpawnSessionState>>,
}

struct SpawnSessionState {
  opening: SpawnSessionId,
  opened: Option<Arc<SpawnSession>>,
}

/// An immutable snapshot that remains valid after the active session changes.
pub struct SpawnSession {
  pub descriptor: SpawnSessionDescriptor,
  pub file: SpawnFile,
}

impl SpawnFileState {
  pub fn new() -> Self {
    Self {
      session: Arc::new(Mutex::new(SpawnSessionState {
        opening: SpawnSessionId::new(),
        opened: None,
      })),
    }
  }

  /// Reserve the next opening before dispatch, leaving the committed file readable if loading fails.
  pub fn begin_open(&self) -> TauriResult<SpawnSessionId> {
    let mut session: MutexGuard<SpawnSessionState> = self.lock()?;

    session.opening = SpawnSessionId::new();

    Ok(session.opening)
  }

  /// Replace file, path, and header together, only for the newest opening.
  pub fn commit_open(&self, id: SpawnSessionId, path: PathBuf, file: SpawnFile) -> TauriResult<SpawnSessionDescriptor> {
    let descriptor: SpawnSessionDescriptor = SpawnSessionDescriptor {
      id,
      path,
      header: file.header.clone(),
    };
    let opened: Arc<SpawnSession> = Arc::new(SpawnSession {
      descriptor: descriptor.clone(),
      file,
    });
    let mut session: MutexGuard<SpawnSessionState> = self.lock()?;

    if session.opening != id {
      return Err(String::from(
        "The spawn opening was superseded by another open or close",
      ));
    }

    let previous = session.opened.replace(opened);

    drop(session);
    drop(previous);

    Ok(descriptor)
  }

  pub fn get_descriptor(&self) -> TauriResult<Option<SpawnSessionDescriptor>> {
    Ok(self.lock()?.opened.as_ref().map(|opened| opened.descriptor.clone()))
  }

  /// Snapshot the committed file addressed by this read; pending replacements do not invalidate it.
  pub fn require(&self, id: SpawnSessionId) -> TauriResult<Arc<SpawnSession>> {
    let session: MutexGuard<SpawnSessionState> = self.lock()?;
    let opened: &Arc<SpawnSession> = session
      .opened
      .as_ref()
      .ok_or_else(|| String::from("No spawn file is open"))?;

    if opened.descriptor.id != id {
      return Err(String::from(
        "The spawn session has changed; reopen the file before reading it",
      ));
    }

    Ok(Arc::clone(opened))
  }

  /// Invalidate unfinished openings and detach the committed file for disposal outside the lock.
  pub fn close(&self) -> TauriResult<Option<Arc<SpawnSession>>> {
    let mut session: MutexGuard<SpawnSessionState> = self.lock()?;

    session.opening = SpawnSessionId::new();
    let previous = session.opened.take();

    drop(session);

    Ok(previous)
  }

  fn lock(&self) -> TauriResult<MutexGuard<'_, SpawnSessionState>> {
    self
      .session
      .lock()
      .map_err(|error| format!("The spawn session is unavailable: {error}"))
  }
}
