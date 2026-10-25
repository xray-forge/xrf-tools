#![doc = include_str!("../README.md")]

pub(crate) mod project;

pub use project::gamedata_check_result::*;
pub(crate) use project::gamedata_finding_factory::GamedataFindingFactory;
pub use project::gamedata_project::*;
pub use project::gamedata_project_read_options::*;
pub use project::gamedata_project_verify_options::*;
pub use project::gamedata_verification_check_report::*;
pub use project::gamedata_verification_report::*;
pub use project::gamedata_verification_rule::*;
pub use project::gamedata_verification_type::*;
pub use xrf_report::{Finding, Status as GamedataVerificationStatus};
