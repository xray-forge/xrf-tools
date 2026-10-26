use crate::OutputVerbosity;

/// The kind of a workflow message, and the only thing that decides how it is rendered.
///
/// A channel is a value rather than a choice of method, so a message can be carried, reordered,
/// filtered, and rendered by something other than the code that produced it. That is what lets one
/// sink print to a terminal, another retain messages for replay, and a third serialize them.
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
  ///
  /// `Error` and `Failure` reach it at every verbosity, so a failing run is never silent; `Verbose`
  /// reaches it only when asked for; everything else is chatter that `Silent` mutes.
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
