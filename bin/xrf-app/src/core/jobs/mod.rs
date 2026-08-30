//! Identity, exclusion and cancellation for work that outlives one command response.

mod job_conclusion;
mod job_description;
mod job_registry;

#[cfg(test)]
mod tests;

pub use job_description::JobDescription;
pub use job_registry::JobRegistry;
