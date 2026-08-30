use crate::job_progress::JobProgress;
use crate::progress_sink::ProgressSink;

/// Writes each snapshot as a log line.
///
/// What a command-line run reports through, where there is no window to draw a bar in but a long silence is still
/// worth breaking. Pair it with a coarse interval: a terminal reading ten lines a second is a terminal nobody reads.
///
/// Renders the deepest level, since that is the one actually moving, prefixed by the phases above it.
#[derive(Default)]
pub struct LoggingSink;

impl ProgressSink for LoggingSink {
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
      Some(total) => log::info!("{path}: {}/{}", deepest.completed, total),
      // A count with nothing to compare it against, which is what an uncountable phase honestly has.
      None => log::info!("{path}: {}", deepest.completed),
    }
  }
}
