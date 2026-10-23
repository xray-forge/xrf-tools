use crate::core::assets::{AssetChangeEvent, AssetMountState};

/// What follows commits in the backend: told what each did once every followed session moved, and what a derivation
/// landing after it did, before any page is.
pub(crate) trait AssetChangeSubscriber: Send + Sync {
  /// Takes what one commit or landed derivation did, on the thread that did it; `assets` publishes the snapshot it left
  /// and the sessions moved to it.
  fn take_change(&self, assets: &AssetMountState, event: &AssetChangeEvent);
}
