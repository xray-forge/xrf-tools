//! The run's UI log: the webview's records, filtered by the backend's rules and written to the run's UI log file alone.

mod ui_log;
mod ui_log_entry;
mod ui_log_filter;
mod ui_log_writer;

#[cfg(test)]
mod tests;

pub use ui_log::UiLog;
pub use ui_log_entry::UiLogEntry;
