use std::fmt::Display;
use std::sync::Arc;

use colored::Colorize;
use xrf_output::{Output, OutputChannel, OutputOptions, OutputVerbosity};

/// Terminal renderer for live workflow output.
#[derive(Default)]
pub struct TerminalOutput;

impl TerminalOutput {
  /// Creates terminal output configured from the CLI verbosity flags.
  pub fn from_options(is_silent: bool, is_verbose: bool) -> OutputOptions {
    let verbosity: OutputVerbosity = match (is_silent, is_verbose) {
      (true, _) => OutputVerbosity::Silent,
      (false, true) => OutputVerbosity::Verbose,
      (false, false) => OutputVerbosity::Normal,
    };

    OutputOptions::new(Arc::new(Self), verbosity)
  }
}

impl Output for TerminalOutput {
  /// Which stream a channel reaches, and in which colour, is the whole of the terminal's contract:
  /// results go to stdout so they can be piped, and anything about the run itself goes to stderr so
  /// it does not land in what was piped.
  fn write(&self, channel: OutputChannel, message: &dyn Display) {
    match channel {
      OutputChannel::Heading | OutputChannel::Success => println!("{}", message.to_string().green()),
      OutputChannel::Warning => eprintln!("{}", message.to_string().yellow()),
      OutputChannel::Failure => eprintln!("{}", message.to_string().red()),
      OutputChannel::Info | OutputChannel::Verbose => println!("{message}"),
      OutputChannel::Error => eprintln!("{message}"),
    }
  }
}

#[cfg(test)]
mod tests {
  use xrf_output::OutputVerbosity;

  use super::TerminalOutput;

  #[test]
  fn maps_cli_verbosity_flags() {
    assert_eq!(
      TerminalOutput::from_options(true, true).get_verbosity(),
      OutputVerbosity::Silent
    );
    assert_eq!(
      TerminalOutput::from_options(false, true).get_verbosity(),
      OutputVerbosity::Verbose
    );
    assert_eq!(
      TerminalOutput::from_options(false, false).get_verbosity(),
      OutputVerbosity::Normal
    );
  }
}
