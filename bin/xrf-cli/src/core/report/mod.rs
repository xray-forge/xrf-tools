//! A run's account of itself for machines: where it goes, the envelope it is written in, and how a verdict is reached.

mod command_build;
mod command_envelope;
mod command_execution;
mod report_destination;
mod report_verdict;
mod reporting_options;
#[cfg(test)]
mod tests;

pub use command_build::CommandBuild;
pub use command_envelope::CommandEnvelope;
pub use command_execution::CommandExecution;
pub use report_destination::ReportDestination;
pub use report_verdict::ReportVerdict;
pub use reporting_options::ReportingOptions;
