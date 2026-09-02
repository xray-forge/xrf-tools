//! Identity, exclusion and cancellation for work that outlives one command response.

mod job_conclusion;
mod job_description;
mod job_leases;
mod job_phases;
mod job_progress_sink;
mod job_registry;
mod job_start;
mod lease_path;

#[cfg(test)]
mod tests;

pub use job_description::JobDescription;
pub use job_phases::JOB_PHASE_PREPARE;
pub use job_registry::{JobRegistration, JobRegistry};
pub use job_start::JobStart;
pub use lease_path::to_comparable_path;
