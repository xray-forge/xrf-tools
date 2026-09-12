use std::path::PathBuf;

use xrf_db::SpawnFile;

use crate::core::session::{Session, SessionId};
use crate::core::types::TauriResult;
use crate::plugins::spawn::SpawnSessionDescriptor;

/// The spawn editor's document lifecycle, with domain-specific descriptor construction.
#[derive(Clone)]
pub struct SpawnFileState {
  /// Publication itself, in the one vocabulary every plugin addresses an opening by.
  pub session: Session<SpawnSession>,
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

  /// Publishes the parsed file and the descriptor that names it, which are decided together.
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

  /// Restores the descriptor alone, so a reload returns without copying the spawn behind it.
  pub fn get_descriptor(&self) -> TauriResult<Option<SpawnSessionDescriptor>> {
    Ok(self.session.get()?.map(|opened| opened.descriptor.clone()))
  }
}
