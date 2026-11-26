use std::time::Duration;

use crate::event::platform_event_kind::PlatformEventKind;

/// Something the window or a device did, stamped with when it happened.
#[derive(Clone, Debug, PartialEq)]
pub struct PlatformEvent {
  /// When it happened, on SDL's clock: the time since the platform started, finer than the poll that read it.
  pub at: Duration,
  pub kind: PlatformEventKind,
}
