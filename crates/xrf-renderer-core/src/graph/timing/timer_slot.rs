use std::sync::Arc;
use std::sync::atomic::AtomicU8;

/// One frame's timestamps on their way back: the readback buffer, where its map stands, and what each stamp ends.
pub(crate) struct TimerSlot {
  pub buffer: wgpu::Buffer,
  pub state: Arc<AtomicU8>,
  /// The pass each stamp ends, by stamp; `None` for a stamp starting an encode group.
  pub names: Vec<Option<String>>,
}

impl TimerSlot {
  /// Nothing copied, a copy recorded but not submitted, a map asked for, a map done.
  pub const IDLE: u8 = 0;
  pub const RECORDED: u8 = 1;
  pub const MAPPING: u8 = 2;
  pub const MAPPED: u8 = 3;
}
