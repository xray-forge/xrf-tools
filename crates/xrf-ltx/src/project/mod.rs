//! A whole tree of configs: what it holds, and the verify and format passes that walk it.

pub(crate) mod ltx_files_formatter;
pub(crate) mod ltx_format_options;
pub(crate) mod ltx_project;
pub(crate) mod ltx_project_format;
pub(crate) mod ltx_project_format_result;
pub(crate) mod ltx_project_options;
pub(crate) mod ltx_project_verify;
pub(crate) mod ltx_project_verify_result;
pub(crate) mod ltx_read_counters;
pub(crate) mod ltx_verify_options;

#[cfg(test)]
mod tests;

pub use crate::project::ltx_files_formatter::LtxFilesFormatter;
pub use crate::project::ltx_format_options::{LTX_PHASE_CHECK, LTX_PHASE_FORMAT, LtxFormatOptions};
pub use crate::project::ltx_project::LtxProject;
pub use crate::project::ltx_project_format_result::LtxProjectFormatResult;
pub use crate::project::ltx_project_options::LtxProjectOptions;
pub use crate::project::ltx_project_verify_result::LtxProjectVerifyResult;
pub(crate) use crate::project::ltx_read_counters::LtxReadCounters;
pub use crate::project::ltx_read_counters::LtxReadCountersSnapshot;
pub use crate::project::ltx_verify_options::{LTX_PHASE_VERIFY, LtxVerifyOptions};
