//! The application's logs. Every backend record goes to the console and to this run's log file, panics included, so a
//! release build, which has no console, still leaves one behind; the webview's records go to the run's UI log file beside
//! it, and nowhere else.

mod file;
mod line;
mod log_filter;
mod log_setup;
mod log_sink;
mod process_start;
mod ui;

pub use log_setup::LogSetup;
pub use process_start::ProcessStart;
pub use ui::{UiLog, UiLogEntry};
