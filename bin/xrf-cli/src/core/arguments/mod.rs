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

pub use engine_target_argument::EngineTargetArgument;
pub use jobs_argument::JobsArgument;
pub use link_policy_argument::LinkPolicyArgument;
pub use ltx_dialect_argument::LtxDialectArgument;
pub(crate) use optional_argument::OptionalArgument;
pub use reporting_arguments::ReportingArguments;
pub use xray_roots_arguments::XrayRootsArguments;
