//! A run: the handle an operation is given, the levels it enters, how it is cancelled and when it reports.

mod emission_gate;
mod job_handle;
mod job_outcome;
mod job_scope;
mod job_state;
mod level_state;

pub(crate) use emission_gate::EmissionGate;
pub use job_handle::JobHandle;
pub use job_outcome::JobOutcome;
pub use job_scope::JobScope;
pub(crate) use job_state::JobState;
pub(crate) use level_state::LevelState;
