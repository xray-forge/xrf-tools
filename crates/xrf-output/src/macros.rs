//! One formatting macro per channel, exported at the crate root: `xrf_output::info!(output, "...", args)` formats
//! straight into the sink without building a `String` first.

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
