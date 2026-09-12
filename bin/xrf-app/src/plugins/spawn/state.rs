use std::path::PathBuf;
use std::sync::Arc;

use xrf_db::SpawnFile;

use crate::core::session::SessionSnapshot;
use crate::core::session::{Session, SessionId};
use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionDescriptor;

/// The spawn editor's document lifecycle, with domain-specific descriptor construction.
#[derive(Clone)]
pub struct SpawnFileState {
  session: Session<SpawnSession>,
}

/// An immutable snapshot that remains valid after the active session changes.
pub struct SpawnSession {
  pub descriptor: SpawnSessionDescriptor,
  pub file: SpawnFile,
}

impl SpawnFileState {
  pub fn new() -> Self {
    Self {
      session: Session::new("spawn"),
    }
  }

  pub fn begin_open(&self, id: SessionId) -> TauriResult<()> {
    self.session.begin_open(id)
  }

  pub fn commit_open(&self, id: SessionId, path: PathBuf, file: SpawnFile) -> TauriResult<SpawnSessionDescriptor> {
    let descriptor: SpawnSessionDescriptor = SpawnSessionDescriptor {
      session_id: id,
      path,
      header: file.header.clone(),
    };

    self.session.commit_open(
      id,
      SpawnSession {
        descriptor: descriptor.clone(),
        file,
      },
    )?;

    Ok(descriptor)
  }

  pub fn get_descriptor(&self) -> TauriResult<Option<SpawnSessionDescriptor>> {
    Ok(self.session.get()?.map(|opened| opened.descriptor.clone()))
  }

  pub fn require(&self, id: SessionId) -> TauriResult<Arc<SessionSnapshot<SpawnSession>>> {
    self.session.require(id)
  }

  /// The command disposes a large spawn on the execution pool after detaching it.
  pub fn close(&self, ids: &[SessionId]) -> TauriResult<Option<Arc<SessionSnapshot<SpawnSession>>>> {
    self.session.detach(ids)
  }
}
