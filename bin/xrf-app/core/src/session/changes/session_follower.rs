use crate::assets::{AssetFanOut, AssetMountState};
use crate::session::{Session, SessionChange};

/// Sessions the asset state moves to their next snapshot after every commit, whatever value they hold.
pub(crate) trait SessionFollower: Send + Sync {
  /// Moves every opening to the snapshot `fan_out` carries, answering what that did to each that moved; none where
  /// nothing is open or every snapshot is that current already.
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> Vec<SessionChange>;
}

impl<T: Send + Sync + 'static> SessionFollower for Session<T> {
  fn follow_in(&self, assets: &AssetMountState, fan_out: &AssetFanOut) -> Vec<SessionChange> {
    Session::follow_in(self, assets, fan_out).into_iter().collect()
  }
}
