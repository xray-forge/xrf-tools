use std::io::{self, Write};
use std::time::Duration;

use serde::Serialize;
use serde_json::Value;
use xrf_build_info::BuildInfo;
use xrf_error::XrfError;
use xrf_job::ExecutionPlan;
use xrf_utils::{StagedFile, format_path};

use crate::core::command::{CommandError, CommandResult};
use crate::core::report::{CommandBuild, CommandExecution, ReportDestination};

/// One run of one command, as a machine reads it.
///
/// `exitCode` is the verdict: 0 success, 3 a check that failed, 1 or 2 a run that could not finish. Nothing restates it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandEnvelope {
  /// Identity of the binary that produced the run, as recorded when it was compiled.
  build: CommandBuild,
  /// The command path as it was dispatched, so a consumer never splits a string to learn it.
  command: Vec<String>,
  #[serde(with = "xrf_utils::duration_ms")]
  duration: Duration,
  /// Why the run could not finish; absent from a run that finished, whatever its verdict, since a check's failure is
  /// told by the result it deposited.
  #[serde(skip_serializing_if = "Option::is_none")]
  error: Option<String>,
  execution: CommandExecution,
  exit_code: u8,
  /// What the command deposited; absent where it failed before judging anything.
  #[serde(skip_serializing_if = "Option::is_none")]
  result: Option<Value>,
}

impl CommandEnvelope {
  /// The envelope of a run of `command` that ended in `outcome` after `duration`, bounded by `execution`, carrying the
  /// result the command deposited.
  pub fn new(
    build: &BuildInfo,
    command: Vec<String>,
    outcome: &CommandResult,
    duration: Duration,
    execution: &ExecutionPlan,
    result: Option<Value>,
  ) -> Self {
    Self {
      build: CommandBuild::from(build),
      command,
      duration,
      error: match outcome {
        Err(error @ CommandError::Execution(_)) => Some(error.message()),
        Ok(()) | Err(CommandError::CheckFailed { .. }) => None,
      },
      execution: CommandExecution::from(execution),
      exit_code: outcome.as_ref().err().map_or(0, CommandError::exit_code),
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

        StagedFile::write(path, &document).map_err(|error| {
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
