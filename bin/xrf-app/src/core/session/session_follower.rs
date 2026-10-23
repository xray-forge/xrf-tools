use crate::core::assets::AssetMountState;
use crate::core::session::{Session, SessionChange};
use crate::core::types::TauriResult;

/// A session the asset state moves to its next snapshot after every commit, whatever value it holds.
pub(crate) trait SessionFollower: Send + Sync {
  /// Moves the session to the project revision `assets` holds now, answering what that did, or `None` where nothing
  /// is open or the snapshot is current already.
  ///
  /// # Errors
  ///
  /// Returns an error when the session's lock is poisoned.
  fn follow(&self, assets: &AssetMountState) -> TauriResult<Option<SessionChange>>;
}

impl<T: Send + Sync + 'static> SessionFollower for Session<T> {
  fn follow(&self, assets: &AssetMountState) -> TauriResult<Option<SessionChange>> {
    Session::follow(self, assets)
  }
}
