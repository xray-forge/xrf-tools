//! The application's logs, through `xrf-logging`: every backend record goes to the console and to this run's log file,
//! panics included; the webview's records go to the run's UI log file beside it, and nowhere else.

mod app_log_setup;
mod ui;

pub use app_log_setup::AppLogSetup;
pub use ui::{UiLog, UiLogEntry};
