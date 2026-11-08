//! A run's account of itself for machines: where it goes, the envelope it is written in, and how a verdict is reached.

mod command_build;
mod command_envelope;
mod command_execution;
mod report_destination;
mod report_verdict;
mod reporting_options;
#[cfg(test)]
mod tests;

pub(crate) use command_build::CommandBuild;
pub(crate) use command_envelope::CommandEnvelope;
pub(crate) use command_execution::CommandExecution;
pub(crate) use report_destination::ReportDestination;
pub(crate) use report_verdict::ReportVerdict;
pub(crate) use reporting_options::ReportingOptions;
