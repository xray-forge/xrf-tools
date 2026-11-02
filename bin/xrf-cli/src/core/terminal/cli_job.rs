use std::io::IsTerminal;
use std::sync::Arc;
use std::time::Duration;

use xrf_job::JobHandle;
use xrf_output::OutputOptions;

use crate::core::terminal::TerminalProgressSink;

/// The job a long command runs as: reported to a person at the terminal and never cancelled.
///
/// Nothing here can stop a run: the command line installs no signal handler, so the handle is passed for its reporting
/// alone. It is a real handle rather than an inert one because a command that walks a whole project owes a terminal
/// some sign that it is still moving.
pub struct CliJob;

impl CliJob {
  /// How often a long command says where it has got to.
  ///
  /// Coarse next to what a window would use. These are lines in a terminal, and a run reporting ten times a second
  /// would bury everything it said before.
  const PROGRESS_INTERVAL: Duration = Duration::from_secs(2);

  /// A job reporting its progress on stderr at the verbosity of `output`, or nowhere when stderr is not a terminal.
  ///
  /// A redirected or captured stderr is a log someone reads after the run, where a line every two seconds is noise
  /// rather than reassurance, and where it would make two identical runs print differently.
  pub fn for_output(output: &OutputOptions) -> JobHandle {
    if !std::io::stderr().is_terminal() {
      return JobHandle::inert();
    }

    JobHandle::with_interval(Arc::new(TerminalProgressSink::new(output)), Self::PROGRESS_INTERVAL)
  }
}
