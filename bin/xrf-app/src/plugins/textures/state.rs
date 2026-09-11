use std::sync::Arc;

use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

use crate::core::session::{DocumentSession, DocumentSessionId, DocumentSnapshot};
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
  browse: DocumentSession<TextureBrowseSession>,
  comparison: DocumentSession<TextureEncodingSession>,
}

impl TextureState {
  pub fn new() -> Self {
    Self {
      browse: DocumentSession::new("texture browse"),
      comparison: DocumentSession::new("texture comparison"),
    }
  }

  pub fn begin_open(&self, id: DocumentSessionId) -> TauriResult<()> {
    self.browse.begin_open(id)
  }

  pub fn begin_comparison(&self, id: DocumentSessionId) -> TauriResult<()> {
    self.comparison.begin_open(id)
  }

  pub fn get_browse(&self) -> TauriResult<Option<Arc<DocumentSnapshot<TextureBrowseSession>>>> {
    self.browse.get()
  }

  pub fn open_browse(
    &self,
    id: DocumentSessionId,
    opened: TextureBrowseSession,
  ) -> TauriResult<Arc<DocumentSnapshot<TextureBrowseSession>>> {
    self.browse.commit_open(id, opened)
  }

  pub fn close(&self, ids: &[DocumentSessionId]) -> TauriResult<()> {
    self.browse.close(ids)?;
    self.comparison.close(ids)
  }

  pub fn hold_comparison(&self, encodings: TextureEncodingSession) -> TauriResult<()> {
    self.comparison.commit_open(encodings.session_id, encodings)?;

    Ok(())
  }

  pub fn get_comparison(&self, id: DocumentSessionId) -> TauriResult<Arc<DocumentSnapshot<TextureEncodingSession>>> {
    self.comparison.require(id)
  }
}
