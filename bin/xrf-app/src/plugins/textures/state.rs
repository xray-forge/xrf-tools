use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

use crate::core::types::TauriResult;
use crate::plugins::textures::TextureSessionId;
use crate::plugins::textures::catalog::TextureCatalogMode;
use crate::plugins::textures::encoding::TextureEncodingSession;

/// The roots and listing mode restored after a frontend reload.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBrowseSession {
  pub roots: XrayRoots,
  pub mode: TextureCatalogMode,
}

/// Owns browse state and comparison publication under one revision lock.
#[derive(Clone)]
pub struct TextureState {
  session: Arc<Mutex<TextureSession>>,
}

struct TextureSession {
  id: TextureSessionId,
  opened: Option<TextureBrowseSession>,
  encodings: Option<Arc<TextureEncodingSession>>,
}

impl TextureState {
  pub fn new() -> Self {
    Self {
      session: Arc::new(Mutex::new(TextureSession {
        id: TextureSessionId::new(),
        opened: None,
        encodings: None,
      })),
    }
  }

  /// Begin an open or comparison, invalidating old candidates and unfinished publications.
  /// The previous browse roots remain available until an open succeeds or the session closes.
  pub fn begin_session(&self) -> TauriResult<TextureSessionId> {
    let mut session: MutexGuard<TextureSession> = self.lock()?;

    session.id = TextureSessionId::new();
    session.encodings = None;

    Ok(session.id)
  }

  pub fn get_browse(&self) -> TauriResult<Option<TextureBrowseSession>> {
    Ok(self.lock()?.opened.clone())
  }

  /// Commit a completed listing only while its opening still owns the session.
  pub fn open_browse(&self, id: TextureSessionId, opened: TextureBrowseSession) -> TauriResult<()> {
    let mut session: MutexGuard<TextureSession> = self.lock()?;

    Self::require_current(&session, id)?;
    session.opened = Some(opened);

    Ok(())
  }

  /// Release cached bytes and prevent unfinished work from reopening the session.
  pub fn close(&self) -> TauriResult<()> {
    let mut session: MutexGuard<TextureSession> = self.lock()?;

    session.id = TextureSessionId::new();
    session.opened = None;
    session.encodings = None;

    Ok(())
  }

  pub fn hold_comparison(&self, encodings: TextureEncodingSession) -> TauriResult<()> {
    let mut session: MutexGuard<TextureSession> = self.lock()?;

    Self::require_current(&session, encodings.session_id)?;
    session.encodings = Some(Arc::new(encodings));

    Ok(())
  }

  /// Snapshot the exact comparison requested; callers decode or serialize after releasing the lock.
  /// An accepted snapshot remains readable if another command replaces or closes the session.
  pub fn get_comparison(&self, id: TextureSessionId) -> TauriResult<Arc<TextureEncodingSession>> {
    let session: MutexGuard<TextureSession> = self.lock()?;

    Self::require_current(&session, id)?;
    session
      .encodings
      .as_ref()
      .map(Arc::clone)
      .ok_or_else(|| String::from("No encoded texture is held; compare the formats first"))
  }

  fn require_current(session: &TextureSession, id: TextureSessionId) -> TauriResult<()> {
    if session.id != id {
      return Err(String::from(
        "The texture session has changed; compare the formats again",
      ));
    }

    Ok(())
  }

  fn lock(&self) -> TauriResult<MutexGuard<'_, TextureSession>> {
    self
      .session
      .lock()
      .map_err(|error| format!("The texture session is unavailable: {error}"))
  }
}
