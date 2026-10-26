use crate::assets::{AssetChangeEvent, AssetMountState};

/// What follows commits in the backend: told what each did once every followed session moved, and what a derivation
/// landing after it did, before any page is.
pub trait AssetChangeSubscriber: Send + Sync {
  /// Takes what one commit or landed derivation did, in the order they published, on the thread that did it or on one
  /// telling of an earlier change; `assets` publishes the snapshot it left and the sessions moved to it.
  fn take_change(&self, assets: &AssetMountState, event: &AssetChangeEvent);
}
