use std::io::{self, Write};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use xrf_build_info::BuildInfo;
use xrf_error::XrfError;
use xrf_job::ExecutionPlan;
use xrf_utils::{format_path, write_file_staged};

use crate::core::command::{CommandError, CommandResult};
use crate::core::report::{CommandOutcome, ReportDestination};

/// One run of one command, as a machine reads it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope {
  /// Identity of the binary that produced the run, as recorded when it was compiled.
  build: BuildInfo,
  /// The command path as it was dispatched, so a consumer never splits a string to learn it.
  command: Vec<String>,
  #[serde(with = "xrf_utils::duration_ms")]
  duration: Duration,
  error: Option<String>,
  /// The width this run was bounded to, and whether anybody chose it.
  ///
  /// Named for the policy rather than for the `--jobs` flag that sets it: a `job` in this workspace is a tracked unit
  /// of work with progress and cancellation, and a field called `jobs` holding a worker count would be that word
  /// meaning something else in the one document both concepts could appear in.
  execution: ExecutionPlan,
  exit_code: u8,
  outcome: CommandOutcome,
  result: Option<Value>,
}

impl CommandEnvelope {
  /// The envelope of a run of `command` that ended in `outcome` after `duration`, bounded by `execution`, carrying the
  /// result the command deposited.
  pub fn new(
    build: BuildInfo,
    command: Vec<String>,
    outcome: &CommandResult,
    duration: Duration,
    execution: ExecutionPlan,
    result: Option<Value>,
  ) -> Self {
    Self {
      build,
      command,
      duration,
      error: outcome.as_ref().err().map(CommandError::message),
      execution,
      exit_code: outcome.as_ref().err().map_or(0, CommandError::exit_code),
      outcome: CommandOutcome::of(outcome),
      result,
    }
  }

  /// Delivers the envelope, or fails saying why it could not be.
  ///
  /// A requested report is part of what the caller asked for rather than a courtesy, so failing to deliver it fails
  /// the run: a CI job that reads the destination afterwards must never take a previous run's document for this one's
  /// answer.
  pub fn publish(&self, destination: &ReportDestination) -> CommandResult {
    match destination {
      ReportDestination::None => Ok(()),
      ReportDestination::Stdout => {
        // One compact line, so a line-oriented consumer and a log both stay readable.
        let document: Vec<u8> = serde_json::to_vec(self)?;

        Self::write_to_stdout(&document).map_err(|error| {
          CommandError::from(XrfError::new_io_error(
            format!("Failed to write the report to stdout: {error}"),
            error.kind(),
          ))
        })
      }
      ReportDestination::File(path) => {
        // A report is a text file, so it ends with a newline.
        let mut document: Vec<u8> = serde_json::to_vec_pretty(self)?;

        document.push(b'\n');

        write_file_staged(path, &document).map_err(|error| {
          CommandError::from(XrfError::new_io_error(
            format!("Failed to write the report to '{}': {error}", format_path(path)),
            error.kind(),
          ))
        })
      }
    }
  }

  fn write_to_stdout(document: &[u8]) -> io::Result<()> {
    let mut stdout: io::StdoutLock<'_> = io::stdout().lock();

    stdout.write_all(document)?;
    stdout.write_all(b"\n")?;
    stdout.flush()
  }
}
