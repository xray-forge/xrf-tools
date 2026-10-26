//! A whole tree of configs: what it holds, and the verify and format passes that walk it.

pub(crate) mod format;
pub(crate) mod ltx_project;
pub(crate) mod ltx_project_options;
pub(crate) mod ltx_read_counters;
pub(crate) mod ltx_read_counters_snapshot;
pub(crate) mod ltx_resolved_root;
pub(crate) mod verify;

#[cfg(test)]
mod tests;

pub use crate::project::format::{LtxFilesFormatter, LtxFormatOptions, LtxProjectFormatResult};
pub use crate::project::ltx_project::LtxProject;
pub use crate::project::ltx_project_options::LtxProjectOptions;
pub(crate) use crate::project::ltx_read_counters::LtxReadCounters;
pub use crate::project::ltx_read_counters_snapshot::LtxReadCountersSnapshot;
pub(crate) use crate::project::ltx_resolved_root::LtxResolvedRoot;
pub use crate::project::verify::{LtxEntryVerification, LtxProjectVerifyResult, LtxVerifyOptions};
