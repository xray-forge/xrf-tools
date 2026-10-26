use std::fmt::Display;

use crate::OutputChannel;

/// Renders live, user-facing workflow messages.
pub trait Output: Send + Sync {
  fn write(&self, channel: OutputChannel, message: &dyn Display);
}
