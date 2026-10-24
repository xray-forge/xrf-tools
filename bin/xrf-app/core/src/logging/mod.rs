//! The application's log: every record to the console and to this run's log file, panics included, so a release build,
//! which has no console, still leaves one behind.

mod log_file;
mod log_sink;
mod setup_logger;
#[cfg(test)]
mod tests;
mod warn_on_unhandled_command;

pub use setup_logger::setup_logger;
pub use warn_on_unhandled_command::warn_on_unhandled_command;
