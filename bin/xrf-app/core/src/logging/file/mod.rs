//! A run's log files: their names, where they are made, and the runs kept beside them.

mod log_file;
mod log_run;
mod log_run_name;
#[cfg(test)]
mod tests;

pub(crate) use log_file::LogFile;
pub(crate) use log_run::LogRun;
