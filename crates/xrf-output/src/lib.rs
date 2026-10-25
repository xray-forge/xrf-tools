#![doc = include_str!("../README.md")]

mod channel;
mod noop_output;
mod output;
mod output_options;
mod output_sequence;
mod output_slot;
mod output_verbosity;
mod record;
mod recording_output;

pub use channel::OutputChannel;
pub use noop_output::NoopOutput;
pub use output::Output;
pub use output_options::OutputOptions;
pub use output_sequence::OutputSequence;
pub use output_slot::OutputSlot;
pub use output_verbosity::OutputVerbosity;
pub use record::OutputRecord;
pub use recording_output::RecordingOutput;

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
