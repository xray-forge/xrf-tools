use crate::OutputVerbosity;

/// The kind of a workflow message, and the only thing that decides how it is rendered.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum OutputChannel {
  Heading,
  Success,
  Warning,
  Failure,
  Info,
  Error,
  Verbose,
}

impl OutputChannel {
  /// Whether a message on this channel reaches the sink at `verbosity`.
  pub const fn is_visible_at(self, verbosity: OutputVerbosity) -> bool {
    match self {
      Self::Error | Self::Failure => true,
      Self::Verbose => matches!(verbosity, OutputVerbosity::Verbose),
      Self::Heading | Self::Success | Self::Warning | Self::Info => !matches!(verbosity, OutputVerbosity::Silent),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::OutputChannel;
  use crate::OutputVerbosity;

  #[test]
  fn renders_failures_at_every_verbosity() {
    for verbosity in [
      OutputVerbosity::Silent,
      OutputVerbosity::Normal,
      OutputVerbosity::Verbose,
    ] {
      assert!(OutputChannel::Error.is_visible_at(verbosity));
      assert!(OutputChannel::Failure.is_visible_at(verbosity));
    }
  }

  #[test]
  fn mutes_chatter_when_silent() {
    for channel in [
      OutputChannel::Heading,
      OutputChannel::Success,
      OutputChannel::Warning,
      OutputChannel::Info,
      OutputChannel::Verbose,
    ] {
      assert!(!channel.is_visible_at(OutputVerbosity::Silent));
    }
  }

  #[test]
  fn renders_detail_only_when_asked_for() {
    assert!(!OutputChannel::Verbose.is_visible_at(OutputVerbosity::Normal));
    assert!(OutputChannel::Verbose.is_visible_at(OutputVerbosity::Verbose));
    assert!(OutputChannel::Info.is_visible_at(OutputVerbosity::Normal));
  }
}
