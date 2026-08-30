//! Identity, exclusion and cancellation for work that outlives one command response.

mod channel_progress_sink;
mod job_conclusion;
mod job_description;
mod job_registry;

#[cfg(test)]
mod tests;

pub use channel_progress_sink::ChannelProgressSink;
pub use job_description::JobDescription;
pub use job_registry::{JobRegistration, JobRegistry};
