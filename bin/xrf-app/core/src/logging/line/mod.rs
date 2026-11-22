//! How a record reads as a line: env_logger's layout, and the moment it is stamped with, whose second also names a
//! run's files.

mod log_layout;
mod log_timestamp;
#[cfg(test)]
mod tests;

pub(crate) use log_layout::LogLayout;
pub(crate) use log_timestamp::LogTimestamp;
