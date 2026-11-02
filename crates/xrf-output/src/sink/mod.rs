//! Where messages go once a verbosity has let them through: rendered, retained, or discarded.

pub(crate) mod noop_output;
pub(crate) mod output;
pub(crate) mod recording_output;

pub use crate::sink::noop_output::NoopOutput;
pub use crate::sink::output::Output;
pub use crate::sink::recording_output::RecordingOutput;
