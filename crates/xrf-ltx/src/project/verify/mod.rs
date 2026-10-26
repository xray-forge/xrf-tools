//! Judging resolved configs against their schemes and what their dialect said while resolving them.

pub(crate) mod ltx_entry_verification;
pub(crate) mod ltx_project_verify;
pub(crate) mod ltx_project_verify_result;
pub(crate) mod ltx_verify_options;

pub use crate::project::verify::ltx_entry_verification::LtxEntryVerification;
pub use crate::project::verify::ltx_project_verify_result::LtxProjectVerifyResult;
pub use crate::project::verify::ltx_verify_options::LtxVerifyOptions;
