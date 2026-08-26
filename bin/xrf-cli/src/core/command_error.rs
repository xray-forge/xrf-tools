use std::error::Error;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Error as IoError;

use xrf_error::XrfError;

/// Failure of a CLI command, classified for the process exit contract.
///
/// Exit codes are the CLI's machine contract: 0 success, 1 execution failure, 2 clap usage error,
/// 3 check failure. `Execution` means the tool could not do its job (IO, environment, internal).
/// `CheckFailed` means a verify/check command ran and judged its input invalid; only check-type
/// commands may produce it, and IO-level failures inside them stay `Execution` so scripts can tell
/// "fix your data" from "fix your environment". Commands never exit themselves: the command prints
/// finding details, the exit point in `application.rs` prints the single final line via [`Display`].
#[derive(Debug)]
pub enum CommandError {
  Execution(XrfError),
  CheckFailed { findings: usize },
}

impl CommandError {
  pub fn new_check_failed(findings: usize) -> Self {
    Self::CheckFailed { findings }
  }

  pub const fn exit_code(&self) -> u8 {
    match self {
      Self::Execution(_) => 1,
      Self::CheckFailed { .. } => 3,
    }
  }

  /// The failure itself, without the label the terminal puts in front of it.
  ///
  /// This is what a machine-readable report carries: `Error:` is how a terminal line announces a
  /// failure, not part of what failed.
  pub fn message(&self) -> String {
    match self {
      Self::Execution(error) => error.to_string(),
      Self::CheckFailed { findings } => format!("Check failed: {findings} finding(s)"),
    }
  }
}

impl Display for CommandError {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    match self {
      // A check verdict already reads as one; an execution failure needs saying.
      Self::Execution(_) => write!(formatter, "Error: {}", self.message()),
      Self::CheckFailed { .. } => formatter.write_str(&self.message()),
    }
  }
}

impl Error for CommandError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Execution(error) => Some(error),
      Self::CheckFailed { .. } => None,
    }
  }
}

impl From<XrfError> for CommandError {
  fn from(value: XrfError) -> Self {
    Self::Execution(value)
  }
}

impl From<IoError> for CommandError {
  fn from(value: IoError) -> Self {
    Self::Execution(value.into())
  }
}

impl From<serde_json::Error> for CommandError {
  fn from(value: serde_json::Error) -> Self {
    Self::Execution(value.into())
  }
}

// In-process argument parsing (`try_get_matches_from`) is exercised by command tests; production
// parsing exits through clap itself before a command runs.
impl From<clap::Error> for CommandError {
  fn from(value: clap::Error) -> Self {
    Self::Execution(XrfError::new_invalid_error(value.to_string()))
  }
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfError;

  use super::CommandError;

  #[test]
  fn classifies_exit_codes() {
    assert_eq!(CommandError::from(XrfError::new_generic_error("boom")).exit_code(), 1);
    assert_eq!(CommandError::new_check_failed(3).exit_code(), 3);
  }

  #[test]
  fn renders_the_final_line_per_class() {
    assert_eq!(
      CommandError::from(XrfError::new_generic_error("boom")).to_string(),
      "Error: Generic error: boom"
    );
    assert_eq!(
      CommandError::new_check_failed(3).to_string(),
      "Check failed: 3 finding(s)"
    );
  }
}
