use std::sync::Arc;

use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

use crate::core::session::{Session, SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
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

/// Browsing and encoded candidates have independent lifetimes; a failed replacement preserves each last success.
#[derive(Clone)]
pub struct TextureState {
  browse: Session<TextureBrowseSession>,
  comparison: Session<TextureEncodingSession>,
}

impl TextureState {
  pub fn new() -> Self {
    Self {
      browse: Session::new("texture browse"),
      comparison: Session::new("texture comparison"),
    }
  }

  pub fn begin_open(&self, id: SessionId) -> TauriResult<()> {
    self.browse.begin_open(id)
  }

  pub fn begin_comparison(&self, id: SessionId) -> TauriResult<()> {
    self.comparison.begin_open(id)
  }

  pub fn get_browse(&self) -> TauriResult<Option<Arc<SessionSnapshot<TextureBrowseSession>>>> {
    self.browse.get()
  }

  pub fn open_browse(
    &self,
    id: SessionId,
    opened: TextureBrowseSession,
  ) -> TauriResult<Arc<SessionSnapshot<TextureBrowseSession>>> {
    self.browse.commit_open(id, opened)
  }

  pub fn close(&self, ids: &[SessionId]) -> TauriResult<()> {
    self.browse.close(ids)?;
    self.comparison.close(ids)
  }

  pub fn hold_comparison(&self, encodings: TextureEncodingSession) -> TauriResult<()> {
    self.comparison.commit_open(encodings.session_id, encodings)?;

    Ok(())
  }

  pub fn get_comparison(&self, id: SessionId) -> TauriResult<Arc<SessionSnapshot<TextureEncodingSession>>> {
    self.comparison.require(id)
  }
}
