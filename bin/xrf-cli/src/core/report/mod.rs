//! A run's account of itself for machines: where it goes, the envelope it is written in, and how its outcome is named.

mod command_envelope;
mod command_outcome;
mod report_destination;
mod report_verdict;
mod reporting_options;
#[cfg(test)]
mod tests;

pub use command_envelope::CommandEnvelope;
pub use command_outcome::CommandOutcome;
pub use report_destination::ReportDestination;
pub use report_verdict::ReportVerdict;
pub use reporting_options::ReportingOptions;
