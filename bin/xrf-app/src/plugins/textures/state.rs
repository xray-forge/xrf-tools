use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

use crate::core::session::{Session, SessionId};
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
  /// What the tree is listing.
  pub browse: Session<TextureBrowseSession>,
  /// The encodes one comparison weighed, held so a save can write the candidate it reported.
  pub comparison: Session<TextureEncodingSession>,
}

impl TextureState {
  pub fn new() -> Self {
    Self {
      browse: Session::new("texture browse"),
      comparison: Session::new("texture comparison"),
    }
  }

  /// Closes both, because a comparison is only ever about the texture the tree is showing.
  pub fn close(&self, ids: &[SessionId]) -> TauriResult<()> {
    self.browse.close(ids)?;
    self.comparison.close(ids)
  }
}
