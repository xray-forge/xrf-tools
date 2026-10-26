//! Rewriting configs in canonical form, or only judging whether they already are.

pub(crate) mod ltx_files_formatter;
pub(crate) mod ltx_format_options;
pub(crate) mod ltx_project_format;
pub(crate) mod ltx_project_format_result;

pub use crate::project::format::ltx_files_formatter::LtxFilesFormatter;
pub use crate::project::format::ltx_format_options::LtxFormatOptions;
pub use crate::project::format::ltx_project_format_result::LtxProjectFormatResult;
