//! The application's log: every record to the console and to this run's log file, panics included, so a release build,
//! which has no console, still leaves one behind.

mod log_file;
mod log_sink;
mod muted_log_targets;
mod process_start;
mod setup_logger;
#[cfg(test)]
mod tests;

pub use process_start::ProcessStart;
pub use setup_logger::setup_logger;
