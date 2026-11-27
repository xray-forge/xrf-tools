#![doc = include_str!("../README.md")]

pub(crate) mod file;
pub(crate) mod line;
pub(crate) mod log_filter;
pub(crate) mod log_setup;
pub(crate) mod log_sink;
pub(crate) mod process_start;
pub(crate) mod tail;
#[cfg(test)]
mod tests;

pub use crate::file::log_file::LogFile;
pub use crate::file::log_run::LogRun;
pub use crate::line::log_layout::LogLayout;
pub use crate::line::log_timestamp::LogTimestamp;
pub use crate::log_filter::LogFilter;
pub use crate::log_setup::LogSetup;
pub use crate::process_start::ProcessStart;
pub use crate::tail::log_tail::LogTail;
