use std::fmt::Display;

use crate::output_channel::OutputChannel;
use crate::sink::Output;

/// Discards every workflow message.
#[derive(Default)]
pub struct NoopOutput;

impl Output for NoopOutput {
  fn write(&self, _: OutputChannel, _: &dyn Display) {}
}

#[cfg(test)]
mod tests {
  use super::{NoopOutput, Output};
  use crate::output_channel::OutputChannel;

  #[test]
  fn discards_every_message() {
    NoopOutput.write(OutputChannel::Error, &"dropped");
  }
}
