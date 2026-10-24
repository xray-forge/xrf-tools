use std::path::PathBuf;

use xrf_app_core::session::{SessionOpening, SessionScope, SessionScopes, Sessions};
use xrf_app_core::types::TauriResult;
use xrf_spawn::SpawnFile;

use crate::SpawnSessionDescriptor;

/// The spawn editors' documents, each held by the tool that opened it, with domain-specific descriptor construction.
#[derive(Clone)]
pub struct SpawnFileState {
  /// The spawns open, in the one vocabulary every plugin addresses an opening by.
  pub files: Sessions<SpawnSession>,
}

/// An immutable snapshot that remains valid after the active session changes.
pub struct SpawnSession {
  pub descriptor: SpawnSessionDescriptor,
  pub file: SpawnFile,
}

impl SpawnFileState {
  /// Whatever scope `scopes` releases closes the spawns it held.
  pub fn new(scopes: &SessionScopes) -> Self {
    Self {
      files: Sessions::new("spawn", scopes),
    }
  }

  /// Publishes the parsed file and the descriptor that names it, which are decided together.
  pub fn commit_open(
    opening: SessionOpening<SpawnSession>,
    path: PathBuf,
    file: SpawnFile,
  ) -> TauriResult<SpawnSessionDescriptor> {
    let descriptor: SpawnSessionDescriptor = SpawnSessionDescriptor {
      session_id: opening.get_id(),
      path,
      header: file.header.clone(),
    };

    opening.commit(SpawnSession {
      descriptor: descriptor.clone(),
      file,
    })?;

    Ok(descriptor)
  }

  /// Restores the descriptor alone of the newest spawn `scope` holds, so a reload returns without copying the spawn
  /// behind it.
  pub fn restore_descriptor(&self, scope: &SessionScope) -> TauriResult<Option<SpawnSessionDescriptor>> {
    Ok(self.files.restore(scope)?.map(|opened| opened.descriptor.clone()))
  }
}
