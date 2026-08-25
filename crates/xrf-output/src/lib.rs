//! Caller-controlled live output for XRF workflows.

mod channel;
mod options;
mod output;
mod record;
mod sequence;

pub use channel::OutputChannel;
pub use options::{OutputOptions, OutputVerbosity};
pub use output::{NoopOutput, Output, RecordingOutput};
pub use record::OutputRecord;
pub use sequence::{OutputSequence, OutputSlot};

/// Sends a section-heading message through workflow output options.
#[macro_export]
macro_rules! heading {
  ($output:expr, $($arguments:tt)*) => {
    $output.heading(format_args!($($arguments)*))
  };
}

/// Sends a successful-outcome message through workflow output options.
#[macro_export]
macro_rules! success {
  ($output:expr, $($arguments:tt)*) => {
    $output.success(format_args!($($arguments)*))
  };
}

/// Sends a warning message through workflow output options.
#[macro_export]
macro_rules! warning {
  ($output:expr, $($arguments:tt)*) => {
    $output.warning(format_args!($($arguments)*))
  };
}

/// Sends a failed-outcome message through workflow output options.
#[macro_export]
macro_rules! failure {
  ($output:expr, $($arguments:tt)*) => {
    $output.failure(format_args!($($arguments)*))
  };
}

/// Sends an informational message through workflow output options.
#[macro_export]
macro_rules! info {
  ($output:expr, $($arguments:tt)*) => {
    $output.info(format_args!($($arguments)*))
  };
}

/// Sends an error message through workflow output options.
#[macro_export]
macro_rules! error {
  ($output:expr, $($arguments:tt)*) => {
    $output.error(format_args!($($arguments)*))
  };
}

/// Sends a verbose message through workflow output options.
#[macro_export]
macro_rules! verbose {
  ($output:expr, $($arguments:tt)*) => {
    $output.verbose(format_args!($($arguments)*))
  };
}
