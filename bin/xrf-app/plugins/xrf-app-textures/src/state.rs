use serde::{Deserialize, Serialize};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionId, SessionScopes, Sessions};
use xrf_app_core::types::TauriResult;
use xrf_vfs::XrayRoots;

use crate::catalog::TextureCatalogMode;
use crate::encoding::TextureEncodingSession;

/// The roots and listing mode restored after a frontend reload.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBrowseSession {
  pub roots: XrayRoots,
  pub mode: TextureCatalogMode,
}

/// Browsing and encoded candidates have independent lifetimes, each held by the tool that opened it.
#[derive(Clone)]
pub struct TextureState {
  /// What each tree is listing.
  pub browses: Sessions<TextureBrowseSession>,
  /// The encodes each comparison weighed, held so a save can write the candidate it reported.
  pub comparisons: Sessions<TextureEncodingSession>,
  /// The application's asset state, which a save commits through.
  pub assets: AssetMountState,
}

impl TextureState {
  /// Whatever scope `scopes` releases closes the browses and comparisons it held.
  pub fn new(assets: AssetMountState, scopes: &SessionScopes) -> Self {
    Self {
      browses: Sessions::new("texture browse", scopes),
      comparisons: Sessions::new("texture comparison", scopes),
      assets,
    }
  }

  /// Closes the caller's handles of both kinds, because a comparison is only ever about the texture its tree shows.
  pub fn close(&self, ids: &[SessionId]) -> TauriResult<()> {
    self.browses.close(ids)?;
    self.comparisons.close(ids)
  }
}
