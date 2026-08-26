use serde::Serialize;
use serde_json::Value;
use xrf_output::OutputOptions;

use crate::core::generic_command::CommandResult;
use crate::core::reporting::ReportingOptions;

/// What a command is handed, beyond its own parsed arguments.
///
/// Commands neither read the reporting flags nor build an output sink: both are resolved once for
/// the whole process, so every command answers to them identically and none can forget one. What a
/// command adds is its structured result, deposited here rather than returned, so that a run which
/// ends in a failed check still delivers the findings that explain the verdict.
///
/// Parallel work never receives the context. Workers take [`OutputOptions`] clones or
/// [`xrf_output::OutputSequence`] slots, which already order what they say; the thread coordinating
/// them deposits the one result after joining.
pub struct CommandContext {
  is_result_requested: bool,
  output: OutputOptions,
  result: Option<Value>,
}

impl CommandContext {
  pub fn new(reporting: &ReportingOptions) -> Self {
    Self {
      is_result_requested: reporting.destination.is_requested(),
      output: reporting.output.clone(),
      result: None,
    }
  }

  /// Where this run's human-facing messages go.
  pub const fn get_output(&self) -> &OutputOptions {
    &self.output
  }

  /// Records what this command found, for whatever machine-readable report was requested.
  ///
  /// Takes a closure rather than a value so that neither building the payload nor serializing it
  /// happens when nothing will read it: an ordinary human run never pays for a report nobody asked
  /// for, and a caller never repeats that condition at the call site. Call it whenever the payload
  /// can be built - before returning a check verdict as much as before returning success, since a
  /// failed check is exactly when its findings are worth reporting.
  pub fn set_result<T: Serialize>(&mut self, build: impl FnOnce() -> T) -> CommandResult {
    if self.is_result_requested {
      self.result = Some(serde_json::to_value(build())?);
    }

    Ok(())
  }

  /// Takes the deposited result for the envelope, leaving the context empty.
  pub fn take_result(&mut self) -> Option<Value> {
    self.result.take()
  }
}

#[cfg(test)]
mod tests {
  use clap::Command;
  use serde_json::json;
  use xrf_output::OutputVerbosity;

  use super::CommandContext;
  use crate::core::reporting::{ReportingOptions, add_reporting_arguments};

  fn context(arguments: &[&str]) -> CommandContext {
    let matches = add_reporting_arguments(Command::new("xrf-cli").no_binary_name(true))
      .try_get_matches_from(arguments)
      .expect("reporting arguments to parse");

    CommandContext::new(&ReportingOptions::from_matches(&matches))
  }

  #[test]
  fn keeps_the_result_a_command_deposited() {
    let mut context: CommandContext = context(&["--json"]);

    context
      .set_result(|| json!({ "volumes": 2 }))
      .expect("result to serialize");

    assert_eq!(context.take_result(), Some(json!({ "volumes": 2 })));
    assert_eq!(context.take_result(), None, "a taken result does not come back");
  }

  /// The payload is never even built when nothing will read it, which is why the deposit takes a
  /// closure: a command that would pay to assemble one does not, and says so in one call.
  #[test]
  fn does_not_build_a_result_nobody_asked_for() {
    let mut context: CommandContext = context(&[]);
    let mut is_built: bool = false;

    context
      .set_result(|| {
        is_built = true;

        json!({ "volumes": 2 })
      })
      .expect("result to be skipped");

    assert!(!is_built, "a payload nobody reads must not be assembled");
    assert_eq!(context.take_result(), None);
  }

  #[test]
  fn carries_the_output_the_run_resolved() {
    assert_eq!(
      context(&["--silent"]).get_output().get_verbosity(),
      OutputVerbosity::Silent
    );
    assert_eq!(context(&[]).get_output().get_verbosity(), OutputVerbosity::Normal);
  }
}
