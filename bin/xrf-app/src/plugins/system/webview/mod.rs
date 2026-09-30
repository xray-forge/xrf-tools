//! The webview's optional browser capabilities: which ones this run started with, and the choice the next start
//! applies, since the browser is started once a run.
//!
//! - [`webview_options_status`] - both, as the settings show them.

pub mod commands;
pub mod webview_options_status;

pub use webview_options_status::WebviewOptionsStatus;
