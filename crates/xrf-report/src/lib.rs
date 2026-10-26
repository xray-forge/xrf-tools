#![doc = include_str!("../README.md")]

pub(crate) mod check_id;
pub(crate) mod check_report;
pub(crate) mod finding;
pub(crate) mod identifier;
pub(crate) mod identifier_error;
pub(crate) mod report;
pub(crate) mod rule_id;
pub(crate) mod status;

#[cfg(test)]
mod tests;

pub use crate::check_id::CheckId;
pub use crate::check_report::CheckReport;
pub use crate::finding::Finding;
pub use crate::identifier_error::IdentifierError;
pub use crate::report::Report;
pub use crate::rule_id::RuleId;
pub use crate::status::Status;
