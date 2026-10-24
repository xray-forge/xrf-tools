use crate::assets::{AssetFanOut, AssetMountState};
use crate::session::{Session, SessionChange};
use crate::types::TauriResult;

/// Sessions the asset state moves to their next snapshot after every commit, whatever value they hold.
pub trait SessionFollower: Send + Sync {
  /// Moves every opening to the snapshot `fan_out` carries, answering what that did to each that moved; none where
  /// nothing is open or every snapshot is that current already.
  ///
  /// # Errors
  ///
  /// Returns an error when a session's lock is poisoned.
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> TauriResult<Vec<SessionChange>>;
}

impl<T: Send + Sync + 'static> SessionFollower for Session<T> {
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> TauriResult<Vec<SessionChange>> {
    Ok(Session::follow_in(self, assets, fan_out)?.into_iter().collect())
  }
}
