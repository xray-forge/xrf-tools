use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::core::assets::asset_change_order_state::AssetChangeOrderState;
use crate::core::assets::asset_change_ticket::AssetChangeTicket;
use crate::core::assets::asset_fan_out::AssetFanOutStart;

/// The order change events are heard in: the order their changes were published. Each event's delivery runs once every
/// one issued before it ran, on whichever thread finishes the run, so no thread ever waits on another's delivery.
#[derive(Default)]
pub(crate) struct AssetChangeOrder {
  state: Mutex<AssetChangeOrderState>,
}

impl AssetChangeOrder {
  /// Runs `publish`, which publishes a change (answering `true`) or nothing, and takes the next place for its event
  /// under the same lock, so places follow the order changes were published in. `publish` must not wait on the worker
  /// pool, since every delivery and issue waits on it.
  pub(crate) fn issue<T, E>(
    self: &Arc<Self>,
    publish: impl FnOnce() -> Result<(T, bool), E>,
  ) -> Result<(T, Option<AssetChangeTicket>), E> {
    let mut state: MutexGuard<AssetChangeOrderState> = self.lock();
    let (published, is_changed) = publish()?;

    Ok((published, is_changed.then(|| self.take_place(&mut state))))
  }

  /// The next place, for an event no change of its own comes with: a derivation landing after its commit.
  pub(crate) fn issue_next(self: &Arc<Self>) -> AssetChangeTicket {
    self.take_place(&mut self.lock())
  }

  fn take_place(self: &Arc<Self>, state: &mut AssetChangeOrderState) -> AssetChangeTicket {
    state.issued += 1;

    AssetChangeTicket::new(Arc::clone(self), state.issued - 1)
  }

  /// Files a delivery in its place and runs every one whose turn has come, unless another thread is running them,
  /// which then runs this one too.
  pub(crate) fn file(&self, number: u64, delivery: Option<AssetFanOutStart>) {
    let mut state: MutexGuard<AssetChangeOrderState> = self.lock();

    state.ready.insert(number, delivery);

    if state.is_delivering {
      return;
    }

    state.is_delivering = true;

    loop {
      let next: u64 = state.delivered;
      let Some(delivery) = state.ready.remove(&next) else {
        state.is_delivering = false;

        return;
      };

      state.delivered += 1;

      drop(state);

      // A delivery that panics must not leave every later one filed with nobody running them.
      if let Some(delivery) = delivery
        && panic::catch_unwind(AssertUnwindSafe(delivery)).is_err()
      {
        log::error!("Delivering a change event panicked");
      }

      state = self.lock();
    }
  }

  /// The order, whatever a panicking delivery left: its fields only change under the lock.
  fn lock(&self) -> MutexGuard<'_, AssetChangeOrderState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
