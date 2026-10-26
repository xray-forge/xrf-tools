use std::collections::BTreeMap;

use crate::assets::change::asset_change_work::AssetChangeWork;

/// Where the change order stands: the places issued and delivered, and the deliveries filed ahead of their turn.
#[derive(Default)]
pub(crate) struct AssetChangeOrderState {
  pub(crate) issued: u64,
  pub(crate) delivered: u64,
  pub(crate) is_delivering: bool,
  /// Deliveries filed ahead of their turn, by ticket; `None` for a ticket given up without one.
  pub(crate) ready: BTreeMap<u64, Option<AssetChangeWork>>,
}
