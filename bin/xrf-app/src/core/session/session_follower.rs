use crate::core::assets::{AssetFanOut, AssetMountState};
use crate::core::session::{Session, SessionChange};
use crate::core::types::TauriResult;

/// A session the asset state moves to its next snapshot after every commit, whatever value it holds.
pub(crate) trait SessionFollower: Send + Sync {
  /// Moves the session to the snapshot `fan_out` carries, answering what that did, or `None` where nothing is open or
  /// the snapshot is that current already.
  ///
  /// # Errors
  ///
  /// Returns an error when the session's lock is poisoned.
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> TauriResult<Option<SessionChange>>;
}

impl<T: Send + Sync + 'static> SessionFollower for Session<T> {
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> TauriResult<Option<SessionChange>> {
    Session::follow_in(self, assets, fan_out)
  }
}
