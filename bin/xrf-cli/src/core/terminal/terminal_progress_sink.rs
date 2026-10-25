use std::sync::Arc;

use xrf_job::{JobProgress, ProgressSink};
use xrf_output::OutputOptions;

use crate::core::terminal::TerminalOutput;

/// Writes each progress snapshot as one line on stderr, for a person watching a long command.
///
/// Every line goes to stderr, whatever stdout carries, so progress never lands in what a caller piped or in a
/// `--json` document; `--silent` mutes it like any other informational line. Renders the deepest level, since that is
/// the one actually moving, prefixed by the phases above it.
pub struct TerminalProgressSink {
  output: OutputOptions,
}

impl TerminalProgressSink {
  /// Reports at the verbosity of `output`, into stderr rather than wherever `output` writes.
  pub fn new(output: &OutputOptions) -> Self {
    Self {
      output: output.with_output(Arc::new(TerminalOutput::new(false))),
    }
  }
}

impl ProgressSink for TerminalProgressSink {
  fn report(&self, progress: &JobProgress) {
    let Some(deepest) = progress.levels.last() else {
      return;
    };

    let path: String = progress
      .levels
      .iter()
      .map(|level| level.id.as_str())
      .collect::<Vec<&str>>()
      .join(" / ");

    match deepest.total {
      Some(total) => xrf_output::info!(self.output, "{path}: {}/{}", deepest.completed, total),
      // A count with nothing to compare it against, which is what an uncountable phase honestly has.
      None => xrf_output::info!(self.output, "{path}: {}", deepest.completed),
    }
  }
}
