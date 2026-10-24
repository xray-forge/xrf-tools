use xrf_output::OutputOptions;

use crate::core::report::ReportDestination;

/// The output selections one run was started with: where humans read it, and where machines do.
pub struct ReportingOptions {
  pub destination: ReportDestination,
  pub output: OutputOptions,
}
