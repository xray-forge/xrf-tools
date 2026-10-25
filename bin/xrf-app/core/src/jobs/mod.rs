//! Identity, exclusion and cancellation for work that outlives one command response.

mod job_conclusion;
mod job_description;
mod job_ending;
mod job_kind;
mod job_leases;
mod job_phase;
mod job_progress_sink;
mod job_registration;
mod job_registry;
mod job_resource;
mod job_start;
#[cfg(feature = "typescript-bindings")]
mod opaque_json;

#[cfg(test)]
mod tests;

pub use job_description::JobDescription;
pub use job_kind::JobKind;
pub use job_phase::JobPhase;
pub use job_registry::JobRegistry;
pub use job_resource::JobResource;
pub use job_start::JobStart;
