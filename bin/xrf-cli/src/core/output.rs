use std::fmt::Display;

use colored::Colorize;
use xrf_output::{Output, OutputChannel};

/// Which of the process streams a message is written to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TerminalStream {
  Stdout,
  Stderr,
}

/// Terminal renderer for live workflow output.
///
/// Verbosity is not a sink's concern - [`xrf_output::OutputOptions`] has already decided a message
/// is visible. All this decides is which stream carries it, and in which colour.
pub struct TerminalOutput {
  is_stdout_available: bool,
}

impl TerminalOutput {
  /// Renders to a terminal whose stdout is free for results, or holds every channel back to stderr
  /// when stdout carries the machine-readable document instead.
  pub const fn new(is_stdout_available: bool) -> Self {
    Self { is_stdout_available }
  }

  /// Which stream a channel reaches, which is the whole of the terminal's contract.
  ///
  /// Results go to stdout so they can be piped, and anything about the run itself goes to stderr so
  /// it does not land in what was piped. When a caller asked for JSON on stdout there is no result
  /// stream left to lend: one human byte there would corrupt the document, so results join the rest
  /// of the run's account of itself on stderr rather than being dropped.
  const fn stream_of(&self, channel: OutputChannel) -> TerminalStream {
    match channel {
      OutputChannel::Heading | OutputChannel::Success | OutputChannel::Info | OutputChannel::Verbose => {
        if self.is_stdout_available {
          TerminalStream::Stdout
        } else {
          TerminalStream::Stderr
        }
      }
      OutputChannel::Warning | OutputChannel::Failure | OutputChannel::Error => TerminalStream::Stderr,
    }
  }

  fn write_line(&self, channel: OutputChannel, message: &dyn Display) {
    match self.stream_of(channel) {
      TerminalStream::Stdout => println!("{message}"),
      TerminalStream::Stderr => eprintln!("{message}"),
    }
  }
}

impl Default for TerminalOutput {
  fn default() -> Self {
    Self::new(true)
  }
}

impl Output for TerminalOutput {
  fn write(&self, channel: OutputChannel, message: &dyn Display) {
    match channel {
      OutputChannel::Heading | OutputChannel::Success => self.write_line(channel, &message.to_string().green()),
      OutputChannel::Warning => self.write_line(channel, &message.to_string().yellow()),
      OutputChannel::Failure => self.write_line(channel, &message.to_string().red()),
      OutputChannel::Info | OutputChannel::Verbose | OutputChannel::Error => self.write_line(channel, message),
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_output::OutputChannel;

  use super::{TerminalOutput, TerminalStream};

  const RESULT_CHANNELS: [OutputChannel; 4] = [
    OutputChannel::Heading,
    OutputChannel::Success,
    OutputChannel::Info,
    OutputChannel::Verbose,
  ];

  const RUN_CHANNELS: [OutputChannel; 3] = [OutputChannel::Warning, OutputChannel::Failure, OutputChannel::Error];

  #[test]
  fn pipes_results_through_stdout_when_it_is_free() {
    let output: TerminalOutput = TerminalOutput::new(true);

    for channel in RESULT_CHANNELS {
      assert_eq!(output.stream_of(channel), TerminalStream::Stdout, "{channel:?}");
    }
  }

  #[test]
  fn keeps_the_account_of_the_run_off_stdout() {
    for is_stdout_available in [true, false] {
      let output: TerminalOutput = TerminalOutput::new(is_stdout_available);

      for channel in RUN_CHANNELS {
        assert_eq!(output.stream_of(channel), TerminalStream::Stderr, "{channel:?}");
      }
    }
  }

  #[test]
  fn leaves_stdout_to_the_document_when_json_was_asked_for() {
    let output: TerminalOutput = TerminalOutput::new(false);

    for channel in RESULT_CHANNELS {
      assert_eq!(output.stream_of(channel), TerminalStream::Stderr, "{channel:?}");
    }
  }
}
