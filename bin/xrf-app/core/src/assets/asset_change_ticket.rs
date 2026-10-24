use std::sync::Arc;

use crate::assets::asset_change_order::AssetChangeOrder;

/// An event's place in [`AssetChangeOrder`], given up where it is dropped undelivered, so a failed fan-out never holds
/// back the ones after it.
pub(crate) struct AssetChangeTicket {
  order: Arc<AssetChangeOrder>,
  number: Option<u64>,
}

impl AssetChangeTicket {
  /// The place numbered `number` in `order`.
  pub(crate) fn new(order: Arc<AssetChangeOrder>, number: u64) -> Self {
    Self {
      order,
      number: Some(number),
    }
  }

  /// Runs `delivery` in this ticket's turn, here or on the thread delivering the turns before it.
  pub(crate) fn deliver(mut self, delivery: impl FnOnce() + Send + 'static) {
    if let Some(number) = self.number.take() {
      self.order.file(number, Some(Box::new(delivery)));
    }
  }
}

impl Drop for AssetChangeTicket {
  fn drop(&mut self) {
    if let Some(number) = self.number.take() {
      log::warn!("A change event was given up undelivered; the events after it go out without it");

      self.order.file(number, None);
    }
  }
}
