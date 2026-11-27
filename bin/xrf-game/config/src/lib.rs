#![doc = include_str!("../README.md")]

pub(crate) mod file;
pub(crate) mod issue;
pub(crate) mod read;

#[cfg(test)]
mod tests;

pub use crate::file::config_file::ConfigFile;
pub use crate::file::config_source::ConfigSource;
pub use crate::issue::config_issue::ConfigIssue;
pub use crate::issue::config_issues::ConfigIssues;
pub use crate::read::config_reader::ConfigReader;
pub use crate::read::config_value::ConfigValue;
// A condlist is a value a config holds, read here and judged by the systems that read it.
pub use xrf_ltx::{Condlist, CondlistBranch, CondlistCondition};
