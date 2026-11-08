//! The named groups of arguments commands declare: each a type that declares its flags on a command (`declare`) and
//! reads them back from what was parsed (`of`).

mod engine_target_argument;
mod jobs_argument;
mod link_policy_argument;
mod ltx_dialect_argument;
mod optional_argument;
mod reporting_arguments;
#[cfg(test)]
mod tests;
mod xray_roots_arguments;

pub(crate) use engine_target_argument::EngineTargetArgument;
pub(crate) use jobs_argument::JobsArgument;
pub(crate) use link_policy_argument::LinkPolicyArgument;
pub(crate) use ltx_dialect_argument::LtxDialectArgument;
pub(crate) use optional_argument::OptionalArgument;
pub(crate) use reporting_arguments::ReportingArguments;
pub(crate) use xray_roots_arguments::XrayRootsArguments;
