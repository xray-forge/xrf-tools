use std::fmt::Display;

use crate::OutputChannel;

/// Renders live, user-facing workflow messages.
///
/// One method rather than one per channel, so a new sink is written once and cannot answer some
/// channels while silently dropping others. The message arrives as [`Display`] and not as a
/// rendered [`String`], so a sink that prints it straight through never allocates.
///
/// Verbosity is not a sink's concern: [`crate::OutputOptions`] has already decided a message is
/// visible by the time it arrives here.
pub trait Output: Send + Sync {
  fn write(&self, channel: OutputChannel, message: &dyn Display);
}
