use serde::Serialize;

use crate::core::command::{CommandError, CommandResult};

/// How a run ended, in the machine contract's own words.
///
/// One variant per class the exit contract distinguishes, so a consumer branches on a name instead of on the numeric
/// code. Usage errors have no variant: clap exits before a command runs, so no envelope exists to carry one.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandOutcome {
  CheckFailed,
  ExecutionFailed,
  Success,
}

impl CommandOutcome {
  /// The class `outcome` falls in.
  pub fn of(outcome: &CommandResult) -> Self {
    match outcome {
      Ok(()) => Self::Success,
      Err(CommandError::CheckFailed { .. }) => Self::CheckFailed,
      Err(CommandError::Execution(_)) => Self::ExecutionFailed,
    }
  }
}
