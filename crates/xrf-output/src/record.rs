use std::fmt::Display;

use crate::OutputChannel;

/// One message, rendered and kept.
///
/// The captured form of what [`crate::Output`] is handed live. A sink that writes straight through
/// never builds one, because rendering a message it is about to print would allocate for nothing;
/// only a sink that must outlive the call — replaying later, or serializing — pays for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OutputRecord {
  channel: OutputChannel,
  message: String,
}

impl OutputRecord {
  pub fn new(channel: OutputChannel, message: impl Into<String>) -> Self {
    Self {
      channel,
      message: message.into(),
    }
  }

  /// Renders `message` into an owned record.
  pub fn render(channel: OutputChannel, message: &dyn Display) -> Self {
    Self::new(channel, message.to_string())
  }

  pub const fn get_channel(&self) -> OutputChannel {
    self.channel
  }

  pub fn get_message(&self) -> &str {
    &self.message
  }
}

#[cfg(test)]
mod tests {
  use super::OutputRecord;
  use crate::OutputChannel;

  #[test]
  fn renders_a_message_into_an_owned_record() {
    let record: OutputRecord = OutputRecord::render(OutputChannel::Error, &format_args!("Mesh {} is not valid", 3));

    assert_eq!(record.get_channel(), OutputChannel::Error);
    assert_eq!(record.get_message(), "Mesh 3 is not valid");
  }
}
